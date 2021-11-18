OUT=out

ARCHIVE=$(OUT)/FLECS.core.process.a

SRC=\
	src/process.cpp

HEADER=\
	posix_spawn.h\
	process.h

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
