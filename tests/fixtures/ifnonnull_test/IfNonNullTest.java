public class IfNonNullTest {

    public static int main(String[] args) {
        Object o = new Object();
        if (o != null) {
            return 55;
        }
        return 0;
    }
}
