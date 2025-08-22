/*
 * Copyright 2020 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

import * as config from './config';
import 'sorttable'; // only for its side-effects
import * as stats from './stats';

/**
 * Creates a loading indicator element.
 */
function createLoader(anchor: Element, label: string)
{
    // This implicitly removes any child nodes.
    anchor.textContent = label;

    const loader = document.createElement("span");
    loader.className = "loader";
    for (let i = 0; i < 3; ++i)
    {
        const loaderBox = document.createElement("span");
        loaderBox.className = "loader-box";
        loader.appendChild(loaderBox);
    }
    anchor.appendChild(loader);
}

// OverpassElement represents one result from Overpass.
interface OverpassElement {
    'id': number;
}

// OverpassResult is the result from Overpass.
interface OverpassResult {
    'elements': OverpassElement[];
}

async function onGpsClick()
{
    const gps = document.querySelector("#filter-based-on-position");
    if (!gps) {
        return;
    }
    gps.removeChild(gps.childNodes[0]);

    // Get the coordinates.
    createLoader(gps, stats.getString("str-gps-wait"));
    let latitude = 0;
    let longitude = 0;
    try
    {
        const position = await new Promise<GeolocationPosition>((resolve, reject) => {
            navigator.geolocation.getCurrentPosition(resolve, reject);
        });
        latitude = position.coords.latitude;
        longitude = position.coords.longitude;
    }
    catch (reason)
    {
        gps.textContent = stats.getString("str-gps-error") + reason;
        return;
    }

    // Get the relations that include this coordinate.
    let query = "[out:json] [timeout:425];\n";
    query += "is_in(" + latitude + "," + longitude + ");\n";
    query += "(._;>;);";
    query += "out meta;";
    createLoader(gps, stats.getString("str-overpass-wait"));
    const protocol = location.protocol != "http:" ? "https:" : "http:";
    let url = protocol + "//overpass-api.de/api/interpreter";
    let request = new Request(url, {method : "POST", body : query});
    let overpassJson: OverpassResult;
    try
    {
        const response = await window.fetch(request);
        overpassJson = await response.json();
    }
    catch (reason)
    {
        gps.textContent = stats.getString("str-overpass-error") + reason;
        return;
    }

    // Build a list of relations.
    const relationIds = [];
    const elements = overpassJson.elements;
    for (let i = 0; i < elements.length; i += 1)
    {
        const element = elements[i];
        if (element.id < 3600000000)
        {
            // Not a relation.
            continue;
        }

        relationIds.push(element.id - 3600000000);
    }

    // Now fetch the list of relations we recognize.
    url = config.uriPrefix + "/api/relations.json";
    request = new Request(url);
    createLoader(gps, stats.getString("str-relations-wait"));
    let knownRelations: Array<number>;
    try
    {
        const response = await window.fetch(request);
        knownRelations = await response.json();
    }
    catch (reason)
    {
        gps.textContent = stats.getString("str-relations-error") + reason;
        return;
    }

    // Filter out the relations we don't recognize.
    const knownRelationIds = [];
    for (let i = 0; i < relationIds.length; i += 1)
    {
        const relationId = relationIds[i];
        if (!knownRelations.includes(relationId))
        {
            continue;
        }

        knownRelationIds.push(relationId);
    }

    // Redirect.
    createLoader(gps, stats.getString("str-redirect-wait"));
    url = config.uriPrefix + "/filter-for/relations/" + knownRelationIds.join(",");
    window.location.href = url;
}

async function initGps()
{
    const gps = document.querySelector("#filter-based-on-position");
    if (!gps)
    {
        return;
    }

    const gpsLink = gps.childNodes[0] as HTMLElement;
    gpsLink.onclick = onGpsClick;
}

/**
 * Starts various JSON requests in case some input of a ref vs osm diff is missing (or the other way
 * around).
 */
async function initRedirects()
{
    const tokens = window.location.pathname.split('/');

    const noOsmStreets = document.querySelector("#no-osm-streets");
    if (noOsmStreets)
    {
        noOsmStreets.removeChild(noOsmStreets.childNodes[0]);
        createLoader(noOsmStreets, stats.getString("str-overpass-wait"));
        const relationName = tokens[tokens.length - 2];
        const link = config.uriPrefix + "/streets/" + relationName + "/update-result.json";
        const request = new Request(link);
        try
        {
            const response = await window.fetch(request);
            const osmStreets = await response.json();
            if (osmStreets.error != "")
            {
                throw osmStreets.error;
            }
            window.location.reload();
        }
        catch (reason)
        {
            noOsmStreets.textContent += " " + stats.getString("str-overpass-error") + reason;
        }
        return;
    }

    const noOsmHousenumbers = document.querySelector("#no-osm-housenumbers");
    if (noOsmHousenumbers)
    {
        noOsmHousenumbers.removeChild(noOsmHousenumbers.childNodes[0]);
        createLoader(noOsmHousenumbers, stats.getString("str-overpass-wait"));
        const relationName = tokens[tokens.length - 2];
        const link = config.uriPrefix + "/street-housenumbers/" + relationName + "/update-result.json";
        const request = new Request(link);
        try
        {
            const response = await window.fetch(request);
            const osmHousenumbers = await response.json();
            if (osmHousenumbers.error != "")
            {
                throw osmHousenumbers.error;
            }
            window.location.reload();
        }
        catch (reason)
        {
            noOsmHousenumbers.textContent += " " + stats.getString("str-overpass-error") + reason;
        }
        return;
    }
}

/**
 * Updates an outdated OSM street list for a relation.
 */
async function onUpdateOsmStreets()
{
    const tokens = window.location.pathname.split('/');

    const streets = document.querySelector("#trigger-streets-update");
    if (!streets) {
        return;
    }
    streets.removeChild(streets.childNodes[0]);
    createLoader(streets, stats.getString("str-toolbar-overpass-wait"));
    const relationName = tokens[tokens.length - 2];
    let link = config.uriPrefix + "/streets/" + relationName + "/update-result.json";
    let request = new Request(link);
    try
    {
        let response = await window.fetch(request);
        const osmStreets = await response.json();
        if (osmStreets.error != "")
        {
            throw osmStreets.error;
        }

        link = config.uriPrefix + "/street-housenumbers/" + relationName + "/update-result.json";
        request = new Request(link);
        response = await window.fetch(request);
        const osmHousenumbers = await response.json();
        if (osmHousenumbers.error != "")
        {
            throw osmHousenumbers.error;
        }
        window.location.reload();
    }
    catch (reason)
    {
        streets.textContent += " " + stats.getString("str-toolbar-overpass-error") + reason;
    }
}

/**
 * Updates an outdated OSM house number list for a relation.
 */
async function onUpdateOsmHousenumbers()
{
    const tokens = window.location.pathname.split('/');

    const housenumbers = document.querySelector("#trigger-street-housenumbers-update");
    if (!housenumbers) {
        return;
    }
    createLoader(housenumbers, stats.getString("str-toolbar-overpass-wait"));
    const relationName = tokens[tokens.length - 2];
    const link = config.uriPrefix + "/street-housenumbers/" + relationName + "/update-result.json";
    const request = new Request(link);
    try
    {
        const response = await window.fetch(request);
        const osmHousenumbers = await response.json();
        if (osmHousenumbers.error != "")
        {
            throw osmHousenumbers.error;
        }
        window.location.reload();
    }
    catch (reason)
    {
        housenumbers.textContent += " " + stats.getString("str-toolbar-overpass-error") + reason;
    }
}

/**
 * Updates an outdated invalid-addr-cities list.
 */
async function onUpdateInvalidAddrCities()
{
    const invalidAddrCities = document.querySelector("#trigger-invalid-addr-cities-update");
    if (!invalidAddrCities) {
        return;
    }
    invalidAddrCities.removeChild(invalidAddrCities.childNodes[0]);
    createLoader(invalidAddrCities, stats.getString("str-toolbar-overpass-wait"));
    const link = config.uriPrefix + "/lints/whole-country/invalid-addr-cities/update-result.json";
    const request = new Request(link);
    try
    {
        const response = await window.fetch(request);
        const ret = await response.json();
        if (ret.error != "")
        {
            throw ret.error;
        }
        window.location.reload();
    }
    catch (reason)
    {
        invalidAddrCities.textContent += " " + stats.getString("str-toolbar-overpass-error") + reason;
    }
}

/**
 * Starts various JSON requests in case some input of a ref vs osm diff is outdated.
 */
async function initTriggerUpdate()
{
    const streetHousenumbers = document.querySelector("#trigger-street-housenumbers-update");
    if (streetHousenumbers)
    {
        const streetHousenumbersLink = streetHousenumbers.childNodes[0] as HTMLLinkElement;
        streetHousenumbersLink.onclick = onUpdateOsmHousenumbers;
        streetHousenumbersLink.href = "#";
    }

    const streets = document.querySelector("#trigger-streets-update");
    if (streets)
    {
        const streetsLink = streets.childNodes[0] as HTMLLinkElement;
        streetsLink.onclick = onUpdateOsmStreets;
        streetsLink.href = "#";
    }

    const invalidAddrCities = document.querySelector("#trigger-invalid-addr-cities-update");
    if (invalidAddrCities)
    {
        const link = invalidAddrCities.childNodes[0] as HTMLLinkElement;
        link.onclick = onUpdateInvalidAddrCities;
        link.href = "#";
    }
}

document.addEventListener("DOMContentLoaded", async function() {
    initGps();
    initRedirects();
    initTriggerUpdate();
    stats.initStats();
});

// vim: shiftwidth=4 softtabstop=4 expandtab:
