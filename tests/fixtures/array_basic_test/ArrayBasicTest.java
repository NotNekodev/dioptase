public class ArrayBasicTest {

    public static int main(String[] args) {
        int[] arr = new int[5];
        arr[0] = 10;
        arr[1] = 20;
        arr[2] = arr[0] + arr[1];
        return arr[2] + arr.length; // 30 + 5
    }
}
