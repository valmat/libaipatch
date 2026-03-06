ROOT := $(abspath ..)
RUST_DIR := $(ROOT)/rust
INCLUDE_DIR := $(ROOT)/include
BUILD_DIR := $(ROOT)/bindings/build
CPP_BUILD_DIR := $(BUILD_DIR)/cpp
DMD_BUILD_DIR := $(BUILD_DIR)/dmd
LDC2_BUILD_DIR := $(BUILD_DIR)/ldc2
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

$(CPP_BUILD_DIR) $(DMD_BUILD_DIR) $(LDC2_BUILD_DIR):
	mkdir -p $@

cpp-example: rust-release | $(CPP_BUILD_DIR)
	$(CXX) $(CPP_CXXFLAGS) $(CPP_DIR)/examples/basic.cpp $(LIBAIPATCH) $(NATIVE_LIBS) -o $(CPP_BUILD_DIR)/basic

cpp-test: rust-release | $(CPP_BUILD_DIR)
	$(CXX) $(CPP_CXXFLAGS) $(CPP_DIR)/tests/smoke.cpp $(LIBAIPATCH) $(NATIVE_LIBS) -o $(CPP_BUILD_DIR)/smoke
	$(CPP_BUILD_DIR)/smoke

d-example-dmd: rust-release | $(DMD_BUILD_DIR)
	dmd -i -I$(D_DIR)/source -of$(DMD_BUILD_DIR)/basic $(D_DIR)/examples/basic.d $(LIBAIPATCH) $(DMD_LINK_FLAGS)

d-test-dmd: rust-release | $(DMD_BUILD_DIR)
	dmd -i -I$(D_DIR)/source -of$(DMD_BUILD_DIR)/smoke $(D_DIR)/tests/smoke.d $(LIBAIPATCH) $(DMD_LINK_FLAGS)
	$(DMD_BUILD_DIR)/smoke

d-example-ldc2: rust-release | $(LDC2_BUILD_DIR)
	ldc2 -i -I$(D_DIR)/source -of=$(LDC2_BUILD_DIR)/basic $(D_DIR)/examples/basic.d $(LIBAIPATCH) $(LDC2_LINK_FLAGS)

d-test-ldc2: rust-release | $(LDC2_BUILD_DIR)
	ldc2 -i -I$(D_DIR)/source -of=$(LDC2_BUILD_DIR)/smoke $(D_DIR)/tests/smoke.d $(LIBAIPATCH) $(LDC2_LINK_FLAGS)
	$(LDC2_BUILD_DIR)/smoke

clean:
	rm -rf $(BUILD_DIR)
