volatile int contador = 0;
int main(void) {
    for (;;) {
        contador = contador + 1;
    }
}
