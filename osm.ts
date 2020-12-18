/*
 * Copyright 2020 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

import * as config from './config';
import 'sorttable'; // only for its side-effects
import * as stats from './stats';

function getOsmString(key: string) {
    return document.getElementById(key).getAttribute("data-value");
}

async function onGpsClick()
{
    const gps = document.querySelector("#filter-based-on-position");
    gps.removeChild(gps.childNodes[0]);

    // Get the coordinates.
    gps.textContent = getOsmString("str-gps-wait");
    let latitude = 0;
    let longitude = 0;
    try
    {
        const position = await new Promise<Position>((resolve, reject) => {
            navigator.geolocation.getCurrentPosition(resolve, reject);
        });
        latitude = position.coords.latitude;
        longitude = position.coords.longitude;
    }
    catch (reason)
    {
        gps.textContent = getOsmString("str-gps-error") + reason;
        return;
    }

    // Get the relations that include this coordinate.
    let query = "[out:json] [timeout:425];\n";
    query += "is_in(" + latitude + "," + longitude + ");\n";
    query += "(._;>;);";
    query += "out meta;";
    gps.textContent = getOsmString("str-overpass-wait");
    const protocol = location.protocol != "http:" ? "https:" : "http:";
    let url = protocol + "//overpass-api.de/api/interpreter";
    let request = new Request(url, {method : "POST", body : query});
    let overpassJson = null;
    try
    {
        const response = await window.fetch(request);
        overpassJson = await response.json();
    }
    catch (reason)
    {
        gps.textContent = getOsmString("str-overpass-error") + reason;
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
    url = config.uriPrefix + "/static/relations.json";
    request = new Request(url);
    gps.textContent = getOsmString("str-relations-wait");
    let knownRelations = null;
    try
    {
        const response = await window.fetch(request);
        knownRelations = await response.json();
    }
    catch (reason)
    {
        gps.textContent = getOsmString("str-relations-error") + reason;
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
    gps.textContent = getOsmString("str-redirect-wait");
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

    const gpsLink = <HTMLElement>gps.childNodes[0];
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
        noOsmStreets.textContent += " " + getOsmString("str-overpass-wait")
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
            noOsmStreets.textContent += " " + getOsmString("str-overpass-error") + reason;
        }
        return;
    }

    const noOsmHousenumbers = document.querySelector("#no-osm-housenumbers");
    if (noOsmHousenumbers)
    {
        noOsmHousenumbers.removeChild(noOsmHousenumbers.childNodes[0]);
        noOsmHousenumbers.textContent += " " + getOsmString("str-overpass-wait")
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
            noOsmHousenumbers.textContent += " " + getOsmString("str-overpass-error") + reason;
        }
        return;
    }

    const noRefHousenumbers = document.querySelector("#no-ref-housenumbers");
    if (noRefHousenumbers)
    {
        noRefHousenumbers.removeChild(noRefHousenumbers.childNodes[0]);
        noRefHousenumbers.textContent += " " + getOsmString("str-reference-wait")
        const relationName = tokens[tokens.length - 2];
        const link = config.uriPrefix + "/missing-housenumbers/" + relationName + "/update-result.json";
        const request = new Request(link);
        try
        {
            const response = await window.fetch(request);
            const refHousenumbers = await response.json();
            if (refHousenumbers.error != "")
            {
                throw refHousenumbers.error;
            }
            window.location.reload();
        }
        catch (reason)
        {
            noRefHousenumbers.textContent += " " + getOsmString("str-reference-error") + reason;
        }
        return;
    }
}

/**
 * Updates an outdated OSM house number list for a relation.
 */
async function onUpdateOsmHousenumbers()
{
    const tokens = window.location.pathname.split('/');

    const housenumbers = document.querySelector("#trigger-street-housenumbers-update");
    housenumbers.removeChild(housenumbers.childNodes[0]);
    housenumbers.textContent += " " + getOsmString("str-toolbar-overpass-wait")
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
        housenumbers.textContent += " " + getOsmString("str-toolbar-overpass-error") + reason;
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
        const streetHousenumbersLink = <HTMLLinkElement>streetHousenumbers.childNodes[0];
        streetHousenumbersLink.onclick = onUpdateOsmHousenumbers;
        streetHousenumbersLink.href = "#";
    }
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
document.addEventListener("DOMContentLoaded", async function(event) {
    initGps();
    initRedirects();
    initTriggerUpdate();
    stats.initStats();
});

// vim: shiftwidth=4 softtabstop=4 expandtab:
