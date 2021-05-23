
libinterface.so:interface.cc
	$(CXX) -fPIC -shared -g -o libinterface.so $< $(CFLAGS) -O3


libinterface.a:interface.cc
	$(CXX) -fPIC -c -o interface.o $< $(CFLAGS) -O3
	$(AR) rvs libinterface.a interface.o
	

.PHONY: clean

clean:
	rm -f ./*.o ./*.so
