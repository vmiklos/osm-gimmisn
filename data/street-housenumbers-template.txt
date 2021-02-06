[out:csv(::id, "addr:street","addr:housenumber", "addr:postcode", "addr:place", "addr:housename", "addr:conscriptionnumber", "addr:flats", "addr:floor", "addr:door", "addr:unit", name, ::type)]  [timeout:425];
area(@AREA@)->.searchArea;
(
  nwr["addr:street"](area.searchArea);
  nwr["addr:housenumber"](area.searchArea);
  nwr["addr:postcode"](area.searchArea);
  nwr["addr:conscriptionnumber"](area.searchArea);
);
out body;
