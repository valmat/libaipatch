ROOT := $(abspath ..)
RUST_DIR := $(ROOT)/rust
INCLUDE_DIR := $(ROOT)/include
BUILD_DIR := $(ROOT)/build/bindings
LIBAIPATCH := $(RUST_DIR)/target/release/libaipatch.a
NATIVE_LIBS := -lgcc_s -lutil -lrt -lpthread -lm -ldl -lc

CPP_DIR := $(ROOT)/bindings/cpp
D_DIR := $(ROOT)/bindings/D
CPP_CXXFLAGS := -std=c++17 -Wall -Wextra -pedantic -I$(INCLUDE_DIR) -I$(CPP_DIR)
DMD_LINK_FLAGS := $(addprefix -L,$(NATIVE_LIBS))
LDC2_LINK_FLAGS := $(addprefix -L=,$(NATIVE_LIBS))

.PHONY: all rust-release cpp-example cpp-test d-example-dmd d-test-dmd d-example-ldc2 d-test-ldc2 clean

all: cpp-test d-test-dmd d-test-ldc2

rust-release:
	cd $(RUST_DIR) && cargo build --release

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

cpp-example: rust-release | $(BUILD_DIR)
	$(CXX) $(CPP_CXXFLAGS) $(CPP_DIR)/examples/basic.cpp $(LIBAIPATCH) $(NATIVE_LIBS) -o $(BUILD_DIR)/cpp-basic

cpp-test: rust-release | $(BUILD_DIR)
	$(CXX) $(CPP_CXXFLAGS) $(CPP_DIR)/tests/smoke.cpp $(LIBAIPATCH) $(NATIVE_LIBS) -o $(BUILD_DIR)/cpp-smoke
	$(BUILD_DIR)/cpp-smoke

d-example-dmd: rust-release | $(BUILD_DIR)
	dmd -i -I$(D_DIR)/source -of$(BUILD_DIR)/d-basic-dmd $(D_DIR)/examples/basic.d $(LIBAIPATCH) $(DMD_LINK_FLAGS)

d-test-dmd: rust-release | $(BUILD_DIR)
	dmd -i -I$(D_DIR)/source -of$(BUILD_DIR)/d-smoke-dmd $(D_DIR)/tests/smoke.d $(LIBAIPATCH) $(DMD_LINK_FLAGS)
	$(BUILD_DIR)/d-smoke-dmd

d-example-ldc2: rust-release | $(BUILD_DIR)
	ldc2 -i -I$(D_DIR)/source -of=$(BUILD_DIR)/d-basic-ldc2 $(D_DIR)/examples/basic.d $(LIBAIPATCH) $(LDC2_LINK_FLAGS)

d-test-ldc2: rust-release | $(BUILD_DIR)
	ldc2 -i -I$(D_DIR)/source -of=$(BUILD_DIR)/d-smoke-ldc2 $(D_DIR)/tests/smoke.d $(LIBAIPATCH) $(LDC2_LINK_FLAGS)
	$(BUILD_DIR)/d-smoke-ldc2

clean:
	rm -rf $(BUILD_DIR)
