public class ArrayLoopTest {

    public static int main(String[] args) {
        int[] arr = new int[10];
        for (int i = 0; i < 10; i++) {
            arr[i] = i * 2;
        }
        int sum = 0;
        for (int i = 0; i < 10; i++) {
            sum += arr[i];
        }
        return sum; // 2*(0+1+...+9) = 90
    }
}
