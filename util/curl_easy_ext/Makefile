OUT=out

ARCHIVE=$(OUT)/curl_easy_ext.a

SRC=\
	src/curl_easy_ext.cpp

HEADER=\
	curl_easy_ext.h

OBJ=$(patsubst src/%.cpp,$(OUT)/%.o,$(SRC))

.PHONY: all
all: $(ARCHIVE)

$(OUT)/%.o: src/%.cpp
	@mkdir -p $(OUT)
	#docker run --rm --user $$(id -u) -v $$(readlink -f $$(pwd)/..):$$(readlink -f $$(pwd)/..) -w $$(pwd) flecs-build\
		g++ -c -g -Wall -Wextra -std=c++17 -I $$(pwd)/../.. -o $@ $<

$(ARCHIVE): $(OBJ) $(HEADER)
	ar cr $(ARCHIVE) $(OBJ)

.PHONY: clean
clean:
	rm -rf $(OUT)
