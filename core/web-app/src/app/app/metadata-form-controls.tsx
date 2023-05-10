import { MetadataKeyType } from '@/utils/metadata-key-type';
import { FieldValues, UseFormRegister } from 'react-hook-form';

// Generate Metadata Form Controls
export const getMetadataFormControls = (
  keyTypes: MetadataKeyType[],
  register: UseFormRegister<FieldValues>
) => {
  const metadataFormControls = keyTypes.map((keyType) => {
    return (
      <div className="form-control mb-4 w-full" key={keyType.key}>
        <label className="label">
          <span className="label-text">{keyType.key}</span>
          {/* <span className="label-text-alt">{keyType.description}</span> */}
        </label>
        <input
          type={keyType.type === 'number' ? 'number' : 'text'}
          placeholder=""
          className="input-bordered input w-full"
          {...register(keyType.key, {
            valueAsNumber: keyType.type === 'number',
          })}
        />
      </div>
    );
  });
  return metadataFormControls;
};
