class Animal {

    int speak() {
        return 1;
    }
}

class Dog extends Animal {

    int speak() {
        return 2;
    }
}

public class InvokeVirtualTest {

    public static int main(String[] args) {
        Animal a = new Dog();
        return a.speak();
    }
}
