export type NullLike = null | undefined;
export type Nullish<T> = T | NullLike;

export const isNull = <T>(item: Nullish<T>): item is NullLike => {
  return item == null;
};

export const isNotNull = <T>(item: Nullish<T>): item is T => {
  return item != null;
};

export const skipNulls = <T>(items: Nullish<T>[]): T[] => {
  return items.filter(isNotNull);
};
