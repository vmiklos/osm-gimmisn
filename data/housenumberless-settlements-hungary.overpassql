[out:json] [timeout:425];
area(3600021335)->.searchArea;
(
  node["place"]["place"!="borough"]["ksh_ref"](area.searchArea);
);
out;
