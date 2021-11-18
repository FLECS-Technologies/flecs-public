OUT=out

ARCHIVE=$(OUT)/llhttp_ext.a

SRC=\
	src/llhttp_ext.cpp

HEADER=\
	llhttp_ext.h

OBJ=$(patsubst src/%.cpp,$(OUT)/%.o,$(SRC))

.PHONY: all
all: $(ARCHIVE)

$(OUT)/%.o: src/%.cpp
	@mkdir -p $(OUT)
	#docker run --rm --user $$(id -u) -v $$(readlink -f $$(pwd)/..):$$(readlink -f $$(pwd)/..) -w $$(pwd) flecs-build\
		g++ -c -g -Wall -Wextra -std=c++17 -I $$(pwd)/../.. -o $@ $<

$(ARCHIVE): $(OBJ) $(HEADER)
	cp $$(pwd)/../../external/llhttp-2.1.4/build/libllhttp.a $(ARCHIVE)
	ar r $(ARCHIVE) $(OBJ)

.PHONY: clean
clean:
	rm -rf $(OUT)
