.PHONY: c
c: flunder_client_example_c

flunder_client_example_c: flunder_client_example.c
	$(eval GIT_ROOT := $(shell git rev-parse --show-toplevel))
	@echo GIT_ROOT: $(GIT_ROOT)
	$(CC) -g -Wall -Wextra -Werror \
		-I $(GIT_ROOT) \
		-o $@ \
		-Wl,-rpath $(GIT_ROOT)/out/amd64/flunder \
		-L $(GIT_ROOT)/out/amd64/flunder -lFLECS.flunder \
		$^

.PHONY: cpp
cpp: flunder_client_example_cpp

flunder_client_example_cpp: flunder_client_example.cpp
	$(eval GIT_ROOT := $(shell git rev-parse --show-toplevel))
	@echo GIT_ROOT: $(GIT_ROOT)
	$(CXX) -g -Wall -Wextra -Werror -std=c++17 \
		-I $(GIT_ROOT) \
		-o $@ \
		-Wl,-rpath $(GIT_ROOT)/out/amd64/flunder \
		-L $(GIT_ROOT)/out/amd64/flunder -lFLECS.flunder \
		$^

.PHONY: all
all: c cpp
