[out:csv(::id, name, highway, service, surface, leisure, ::type)]  [timeout:425];
area(@AREA@)->.searchArea;
// all but service, footway, path + avoid bridges and special highways
way(area.searchArea)[highway][name][!"bridge"]["highway"!~"platform|bus_stop|footway|steps|raceway|service|pedestrian|proposed|abandoned|rest_area|path|corridor|via_ferrata"];
out;
// service roads with names, avoid parking ways, drive-through
way(area.searchArea)[highway=service][name][!"bridge"][service!~"parking_aisle|drive-through|slipway"];
out;
// avoid underpass and bridge footways, platforms
way(area.searchArea)["highway"~"footway|steps|pedestrian"][name][layer!~"-"][!"bridge"]["indoor"!="yes"]["public_transport"!="platform"];
out;
// include pedestrian + layer=-1 with valid name
way(area.searchArea)["highway"~"footway|pedestrian"][name][layer~"-"][name!~"aluljáró|passzázs|alagút"][name~"utca|köz| út|sétány|tér|lépcső|fasor"][!"bridge"]["indoor"!="yes"]["public_transport"!="platform"];
out;
// path with valid street name, try to exclude hiking path with comments in name tag
way(area.searchArea)["highway"="path"][name][name~"utca| út|köz|árok|lépcső|gát|part|tér|tere|sétány|park|dűlő|sor|kert"][name!~"garázssor"](area.searchArea);
out;
// include parking aisle with valid street name
way(area.searchArea)["highway"="service"][service="parking_aisle"][name][name~"utca|köz|sétány|tér|tere"](area.searchArea);
out;
// include amenity parking  with valid street name
way(area.searchArea)["amenity"="parking"][!highway][name~" utca$| köz$| sétány$| tér$| tere$| udvar$"][!"addr:street"][name!~"gazdasági udvar"](area.searchArea);
out;
// include parks with names
way(area.searchArea)["leisure"="park"][name];
out;
relation(area.searchArea)["leisure"="park"][name];
out;
// include squares
way(area.searchArea)["place"="square"]["leisure"!="park"]["highway"!="pedestrian"][name];
out;
relation(area.searchArea)["leisure"="square"][name];
out;
// area boundary streets
rel(@RELATION@)->.searchRelation;
way(r.searchRelation)[highway][name][!"bridge"];
out;
// highway=track can also be anonymous, but many mistag based on road surface
