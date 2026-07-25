public class NativeMethodTest {

    static native int native_test();

    public static int main(String[] args) {
        return native_test();
    }
}
