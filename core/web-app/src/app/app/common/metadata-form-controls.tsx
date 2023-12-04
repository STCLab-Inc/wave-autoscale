import { MetadataKeyType } from '@/utils/metadata-key-type';
import { FieldValues, UseFormRegister } from 'react-hook-form';

const createMetadataControl = (
  keyType: MetadataKeyType,
  register: UseFormRegister<FieldValues>
) => (
  <div className="form-control w-full px-4 py-2" key={keyType.key}>
    <label className="label px-0 py-2">
      <span className="text-md label-text px-2">{keyType.key}</span>
      {/* <span className="label-text-alt">label-text-alt</span> */}
    </label>
    <input
      type={keyType.type === 'number' ? 'number' : 'text'}
      placeholder={keyType.key}
      className="input-bordered input my-2 w-full px-4 text-sm focus:outline-none"
      autoComplete="off"
      autoCapitalize="off"
      autoCorrect="off"
      {...register(keyType.key, {
        valueAsNumber: keyType.type === 'number',
      })}
    />
  </div>
);

export const getMetadataFormControls = (
  keyTypes: MetadataKeyType[],
  register: UseFormRegister<FieldValues>
) => {
  const metadataFormControls = keyTypes.map((keyType) =>
    createMetadataControl(keyType, register)
  );
  return metadataFormControls;
};
