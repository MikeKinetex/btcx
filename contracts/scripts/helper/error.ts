export class BaseError extends Error {
  protected constructor(name: string, message: string) {
    super(message);
    this.name = name;
  }
}
