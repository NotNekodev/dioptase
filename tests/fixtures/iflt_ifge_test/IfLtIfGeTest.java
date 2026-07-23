public class IfLtIfGeTest {

    public static int main(String[] args) {
        int x = -3;
        if (x < 0) {
            x = 0 - x;
        }
        return x; // abs(-3) = 3
    }
}
