public class PutStaticTest {

    static int value = 0;

    static void bump() {
        value = value + 10;
    }

    public static int main(String[] args) {
        bump();
        bump();
        bump();
        return value; // 0 + 10 + 10 + 10
    }
}
