[out:csv(::type, ::id, name)] [timeout:425];
area(3600021335)->.searchArea;
(
  relation["boundary"="administrative"]["admin_level"="8"](area.searchArea);
);
out;
