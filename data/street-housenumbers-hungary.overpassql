[out:csv("addr:postcode","addr:city", "addr:street", "addr:housenumber", ::user, ::id, ::type, ::timestamp, "addr:place", "addr:unit", "name", "fixme")] [timeout:425];
area(3600021335)->.searchArea;
(
  node["addr:housenumber"](area.searchArea);
  way["addr:housenumber"](area.searchArea);
  relation["addr:housenumber"](area.searchArea);
);
out meta;
