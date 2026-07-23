public class ClinitTest {

    static int a = compute();

    static int compute() {
        return 20 + 22;
    }

    public static int main(String[] args) {
        return a;
    }
}
