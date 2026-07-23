public class GotoTest {

    public static int main(String[] args) {
        int i = 0;
        int sum = 0;
        while (i < 5) {
            sum += i;
            i++;
        }
        return sum; // 0+1+2+3+4 = 10
    }
}
