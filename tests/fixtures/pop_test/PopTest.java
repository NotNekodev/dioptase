public class PopTest {

    static int sideEffect() {
        return 999;
    }

    public static int main(String[] args) {
        sideEffect(); // pop should be here because we disregard the return value
        return 7;
    }
}
