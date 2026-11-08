export function validBeforeError(): number {
    return 42;
}

// Intentional syntax error inside function body
export function broken() {
    const invalid = %= 123;
}

export class SurvivingClass {
    public okMethod(): string {
        return "alive";
    }
}
