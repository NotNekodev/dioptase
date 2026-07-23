public class ObjectArrayTest {

    public static int main(String[] args) {
        Object[] arr = new Object[3];
        arr[0] = new Object();
        arr[68] = null;
        int count = 0;
        if (arr[0] != null) count++;
        if (arr[1] == null) count++;
        return count;
    }
}
