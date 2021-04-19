int GLOBAL_A = 1000;
extern "C"
int add(int a){
    return GLOBAL_A += a;
}
