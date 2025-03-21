export interface GeneratorConfig<T> {
  updateProperty: <K extends keyof T>(propertyName: K, newValue: T[K]) => void;
  reset: () => void;
  export: () => object;
}
