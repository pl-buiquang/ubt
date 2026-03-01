import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class HelloTest {
    @Test
    void testGreet() {
        assertEquals("Hello, World!", Hello.greet());
    }
}
