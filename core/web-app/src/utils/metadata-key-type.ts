// https://stackoverflow.com/questions/51465182/typescript-get-keys-of-a-generic-interface-without-using-enum
// type KeyTypes<T> = { [K in keyof T]: { key: K; type: T[K] } }[keyof T][];

export type MetadataKeyType = { key: string; type: string };

export function getInterfaceKeyTypes<T extends {}>(obj: T): MetadataKeyType[] {
  const keys = Object.keys(obj) as (keyof T)[];
  return keys.map((key) => ({
    key: key as string,
    type: typeof obj[key],
  })) as MetadataKeyType[];
}
