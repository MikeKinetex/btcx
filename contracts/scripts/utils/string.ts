export const stringList = (value: string): string[] => {
  if (!value) {
    return [];
  }

  const items = value.split(',');
  return items;
};
