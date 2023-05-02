'use client';

import {
  KeyType,
  generateMetricDefinition,
  getMetricKeyTypes,
} from '@/utils/bindings';
import { useRouter } from 'next/navigation';
import { useMemo, useState } from 'react';
import { FieldValues, UseFormRegister, useForm } from 'react-hook-form';

// Metric Types
const metricKeyTypes = getMetricKeyTypes();
const metricOptions = metricKeyTypes.map((metricKeyType) => (
  <option key={metricKeyType.metricName} value={metricKeyType.metricName}>
    {metricKeyType.metricName}
  </option>
));

// Generate Metadata Form Controls
const getMetadataFormControls = (
  keyTypes: KeyType[],
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

export default function MetricDetailDrawer() {
  const {
    register,
    handleSubmit,
    watch,
    formState: { errors },
  } = useForm();
  const router = useRouter();

  const [selectedMetricType, setSelectedMetricType] = useState<string>();

  //
  // Events
  //
  const onClickOverlay = () => {
    let path = window.location.href;
    path = path.slice(0, path.lastIndexOf('/'));
    router.push(path);
  };

  const onChangeMetricType = (e: any) => {
    const metricType = e.target.value;
    setSelectedMetricType(metricType);
  };

  const metadataFormControls = useMemo(() => {
    if (selectedMetricType) {
      const keyTypes = metricKeyTypes.find(
        (metricKeyType) => metricKeyType.metricName === selectedMetricType
      )?.keyTypes;
      if (keyTypes) {
        return getMetadataFormControls(keyTypes, register);
      }
    }
  }, [selectedMetricType]);

  const onSubmit = (data: any) => {
    const { id, metric_kind, ...metadata } = data;
    const metricDefinition = generateMetricDefinition({
      id,
      metric_kind,
      metadata,
    });
    console.log({ metricDefinition });
  };

  return (
    <div className="drawer drawer-end fixed inset-y-0 bottom-0 top-16 z-50">
      <input id="drawer" type="checkbox" className="drawer-toggle" checked />
      <div className="drawer-side">
        <label
          htmlFor="drawer"
          className="drawer-overlay"
          onClick={onClickOverlay}
        />
        <div className="drawer-content w-[32rem] overflow-y-auto bg-base-100 p-4">
          <form className="" onSubmit={handleSubmit(onSubmit)}>
            <div className="mb-4 flex items-center justify-between">
              <h2 className="font-bold">Metric</h2>
              <button type="submit" className="btn-primary btn-sm btn">
                Save
              </button>
            </div>
            <div className="form-control mb-4 w-full">
              <label className="label">
                <span className="label-text">ID</span>
                <span className="label-text-alt">used as a variable name</span>
              </label>
              <input
                type="text"
                placeholder="Type here"
                className="input-bordered input w-full"
                {...register('id', { required: true })}
              />
            </div>
            <div className="form-control mb-4 w-full">
              <label className="label">
                <span className="label-text">Metric Type</span>
                <span className="label-text-alt"></span>
              </label>
              <select
                className="select-bordered select"
                defaultValue="Pick one"
                {...register('metric_kind', {
                  required: true,
                  onChange: onChangeMetricType,
                })}
              >
                <option disabled>Pick one</option>
                {metricOptions}
              </select>
            </div>
            {metadataFormControls}
          </form>
        </div>
      </div>
    </div>
  );
}
