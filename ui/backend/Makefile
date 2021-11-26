OUT=out

BINARY=$(OUT)/flecs-backend

DEPENDS=\
	llhttp_ext

SRC=\
	src/main.cpp\
	src/http_request_handler.cpp

OBJ=$(patsubst src/%.cpp,$(OUT)/%.o,$(SRC))

HEADER=\
	http_request_handler.h

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
		g++ -c -g -Wall -Wextra -std=c++17 -fPIE -I $$(pwd)/../.. -o $@ $<

$(BINARY): $(OBJ) $(HEADER)
	#docker run --rm --user $$(id -u) -v $$(readlink -f $$(pwd)/..):$$(readlink -f $$(pwd)/..) -w $$(pwd) flecs-build\
		g++ -g -Wall -Wextra -std=c++17 -fPIE -I $$(pwd)/../.. -o $@ $(OBJ)\
		$$(pwd)/../../util/llhttp_ext/out/llhttp_ext.a $$(pwd)/../../util/curl_easy_ext/out/curl_easy_ext.a $$(pwd)/../../core/process/out/FLECS.core.process.a\
		-ljsoncpp

.PHONY: clean
clean:
	rm -rf $(OUT)
