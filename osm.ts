/*
 * Copyright 2020 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

import * as config from './config';
// eslint-disable-next-line @typescript-eslint/no-unused-vars
import * as sorttable from 'sorttable'; // only for its side-effects

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

// eslint-disable-next-line @typescript-eslint/no-unused-vars
document.addEventListener("DOMContentLoaded", async function(event) {
    const gps = document.querySelector("#filter-based-on-position");
    if (!gps)
    {
        return;
    }

    const gpsLink = <HTMLElement>gps.childNodes[0];
    gpsLink.onclick = onGpsClick;
});

// vim: shiftwidth=4 softtabstop=4 expandtab:
