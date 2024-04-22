export const isArray = <I, O>(items: readonly I[] | O): items is readonly I[] => {
  return Array.isArray(items);
};
