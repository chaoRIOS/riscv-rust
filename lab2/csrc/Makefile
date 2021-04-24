WORK_DIR = /home/ychen/
DRAMSIM_PATH = $(WORK_DIR)/DRAMSim2
DRAMSIM_LIB = $(DRAMSIM_PATH)/libdramsim.so
	
test:cfoo.cpp
	g++ -fPIC -shared -o libcfoo.so cfoo.cpp


#tell the linker the rpath so that we don't have to muck with LD_LIBRARY_PATH, etc
dramsim_interface: dramsim_interface.cpp
	$(CXX) -fPIC -shared -g -o libdramsim_interface.so dramsim_interface.cpp -I$(DRAMSIM_PATH) -L$(DRAMSIM_PATH) -ldramsim -Wl,-rpath=$(DRAMSIM_PATH)
