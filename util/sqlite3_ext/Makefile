OUT=out

ARCHIVE=$(OUT)/sqlite3_ext.a

SRC=\
	src/sqlite3_db.cpp

HEADER=\
	sqlite3_db.h

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
