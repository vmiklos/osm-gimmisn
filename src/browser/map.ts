/*
 * Copyright 2026 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

import * as geojson from "geojson";
import * as L from "leaflet";

// The property of a feature we show as a popup.
interface NameSupplier {
    name: string | null;
}

// Shows a progress indicator at the center of the screen, till the fetch is in progress.
function createLoader(): Element
{
    const loader = document.createElement("div");
    loader.className = "loader";
    // osm.css styles the loader for inline, in-page use; center it above the fullscreen map here.
    loader.style.position = "absolute";
    loader.style.top = "50%";
    loader.style.left = "50%";
    loader.style.transform = "translate(-50%, -50%)";
    loader.style.zIndex = "1000";
    for (let i = 0; i < 3; i += 1)
    {
        const loaderBox = document.createElement("span");
        loaderBox.className = "loader-box";
        loader.appendChild(loaderBox);
    }
    document.body.appendChild(loader);
    return loader;
}

// Binds the name of a feature as its popup, if any.
function onEachFeature(
    feature: geojson.Feature<geojson.GeometryObject, NameSupplier>,
    layer: L.Layer
)
{
    if (feature.properties == null || feature.properties.name == null)
    {
        return;
    }

    layer.bindPopup(feature.properties.name);
}

// Shows the geojson referenced by the "geojson" query parameter on a fullscreen leaflet map.
async function initMap(): Promise<void>
{
    if (!document.getElementById("map"))
    {
        // Not on the map page.
        return;
    }

    const map = L.map("map");
    map.attributionControl.setPrefix(
        '<a href="https://leafletjs.com" title="A JavaScript library for interactive maps">Leaflet</a>'
    );

    L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
        attribution:
            '&copy; <a href="https://www.openstreetmap.org/copyright" target="_blank">OpenStreetMap</a> contributors.',
    }).addTo(map);

    const urlParams = new URLSearchParams(window.location.search);
    const geojsonURL = urlParams.get("geojson");
    if (geojsonURL == null)
    {
        return;
    }

    const loader = createLoader();
    try
    {
        const response = await window.fetch(geojsonURL);
        const featureCollection = await response.json();
        const geoJSON = L.geoJSON(featureCollection, {
            onEachFeature: (feature, layer) => onEachFeature(feature, layer),
        }).addTo(map);
        map.fitBounds(geoJSON.getBounds());
    }
    finally
    {
        loader.remove();
    }
}

export { initMap };

// vim: shiftwidth=4 softtabstop=4 expandtab:
