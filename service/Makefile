OUT=out

BINARY=$(OUT)/flecs

SRC=\
	main.cpp\
	src/app.cpp\
	src/app_db.cpp\
	src/service_app_manager.cpp\
	src/service_help.cpp\
	src/service_rpc.cpp\
	src/service.cpp

OBJ=$(patsubst src/%.cpp,$(OUT)/%.o,$(SRC))

HEADER=\
	app_db.h\
	service_app_manager.h\
	service_factory.h\
	service_help.h\
	service_rpc.h\
	service_table.h\
	service.h

.PHONY: help
help:
	@echo OUT: $(OUT)
	@echo SRC: $(SRC)
	@echo OBJ: $(OBJ)
	@echo HEADER: $(HEADER)

.PHONY: all
all: $(BINARY)

$(OUT)/%.o: src/%.cpp
	@mkdir -p $(OUT)
	#docker run --rm --user $$(id -u) -v $$(readlink -f $$(pwd)/..):$$(readlink -f $$(pwd)/..) -w $$(pwd) flecs-build\
		g++ -c -g -Wall -Wextra -Wfatal-errors -std=c++17 -I $$(pwd)/.. -o $@ $<

$(BINARY): $(OBJ)
	#docker run --rm --user $$(id -u) -v $$(readlink -f $$(pwd)/..):$$(readlink -f $$(pwd)/..) -w $$(pwd) flecs-build\
		g++ -g -Wall -Wextra -std=c++17 -I $$(pwd)/.. -o $@ $(OBJ)\
		$$(pwd)/../util/process/out/FLECS.core.process.a\
		$$(pwd)/../util/curl_easy_ext/out/curl_easy_ext.a\
		$$(pwd)/../util/sqlite3_ext/out/sqlite3_ext.a\
		$$(pwd)/../external/yaml-cpp-0.7.0/build/libyaml-cpp.a\
		$$(pwd)/../external/curl-7.80.0/lib/.libs/libcurl.a\
		-lssl -lcrypto -lssl -lcrypto -lz\
		-lsqlite3
