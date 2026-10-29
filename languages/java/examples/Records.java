record Empty() {}

record Point(int x, int y) {}

record Collision(int Collision) {
    Collision() { this(0); }
}

record Values<T>(@Deprecated T value, java.util.List<T> items, int[] numbers) {
    public Values(T value, java.util.List<T> items, int[] numbers) {
        this.value = value;
        this.items = items;
        this.numbers = numbers;
    }

    public T value() { return value; }
    public int numbers(int index) { return numbers[index]; }
    public void consume(String argument) {}
}

class Holder {
    private int ordinary;
    public void method(int argument) {}
    record Nested(String name) {}
}

record Messages(String... messages) {}
