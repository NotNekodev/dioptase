public class LdcTest {

    public static int main(String[] args) {
        int x = 40042; // exceeds sipush range -> forces ldc
        return x % 256; // 40042 % 256 = 106
    }
}
