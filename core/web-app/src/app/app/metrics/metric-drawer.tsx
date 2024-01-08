import React, { useEffect } from 'react';
import { Controller, useForm } from 'react-hook-form';
import Image from 'next/image';
import MetricService from '@/services/metric';
import { generateMetricDefinition } from '@/utils/metric-binding';
import { MetricDefinition } from '@/types/bindings/metric-definition';
import yaml from 'js-yaml';
import {
  DEFINITION_ID_RULE_DESCRIPTION,
  transformDefinitionId,
} from '@/utils/definition-id';
import { METRIC_COLLECTORS } from '@/utils/metric-collector';
import dynamic from 'next/dynamic';

const YAMLEditor = dynamic(() => import('../common/yaml-editor'), {
  ssr: false,
});

const METRIC_COLLECTOR_OPTIONS = [
  {
    name: 'Select a metric collector',
    value: '',
    description: '',
  },
  ...METRIC_COLLECTORS,
].map((collector) => (
  <option key={collector.value} value={collector.value}>
    {collector.name} ({collector.description})
  </option>
));

export default function MetricDetailDrawer({
  metric,
  onClose,
  onChange,
}: {
  metric?: MetricDefinition;
  onClose: () => void;
  onChange: (fetchFlag: boolean) => void;
}) {
  const {
    formState: { isValid },
    register,
    handleSubmit,
    control,
    setValue,
    reset,
  } = useForm();

  const dbId = metric?.db_id;
  const isNew = !dbId;

  useEffect(() => {
    if (isNew) {
      reset();
      return;
    }

    const { kind, db_id, id, collector, metadata, enabled, ...rest } = metric;
    setValue('kind', kind);
    setValue('db_id', db_id);
    setValue('id', id);
    setValue('enabled', enabled);
    setValue('collector', collector);
    let metadataYaml = '';
    if (Object.keys(metadata).length > 0) {
      metadataYaml = yaml.dump(metadata);
    }
    setValue('metadata', metadataYaml);
  }, [metric]);

  const onClickInitialize = async () => {
    reset();
  };

  const onClickRemove = async () => {
    if (isNew) {
      return;
    }
    if (!confirm('Are you sure you want to delete this metric?')) {
      return;
    }
    try {
      const response = await MetricService.deleteMetric(dbId);
      console.info({ response });

      onChange(true);
      onClose();
    } catch (error) {
      console.error(error);
    }
  };

  const onSubmit = async (data: any) => {
    const { kind, db_id, id, collector, metadata, enabled, ...rest } = data;

    if (!isValid) {
      alert('Please check the form again.');
      return;
    }
    const metricsItem = generateMetricDefinition({
      kind: 'Metric',
      id: id,
      db_id: dbId,
      collector: collector,
      enabled,
      metadata: yaml.load(metadata ?? ''),
    });
    try {
      if (isNew) {
        const response = await MetricService.createMetric(metricsItem);
      } else {
        const response = await MetricService.updateMetric(metricsItem);
      }
      onChange(true);
      onClose();
    } catch (error) {
      console.error(error);
    }
  };

  return (
    <div className="metric-drawer drawer drawer-end fixed bottom-0 right-0 top-16 z-20 w-full">
      <input
        id="drawer"
        type="checkbox"
        className="drawer-toggle"
        defaultChecked={true}
      />
      <div className="drawer-side h-full border-t border-gray-200">
        <label
          htmlFor="drawer"
          className="drawer-overlay"
          onClick={() => onClose()}
        />
        <div className="drawer-content flex h-full min-w-[48rem] max-w-[48rem] flex-col overflow-y-auto border-l border-gray-200 bg-base-100 pb-20">
          <form className="" onSubmit={handleSubmit(onSubmit)}>
            <div className="flex h-14 min-h-14 w-full min-w-full flex-row items-center justify-between border-b border-dashed border-gray-400 bg-gray-75">
              <span className="font-Pretendard truncate whitespace-nowrap px-4 text-lg font-semibold text-gray-1000">
                Metric
              </span>
              <div className="flex px-4">
                {isNew ? (
                  <button
                    className="ml-1 mr-1 flex h-8 items-center justify-center rounded-md border border-red-400 bg-red-400 pl-5 pr-5 text-sm text-gray-50"
                    onClick={onClickInitialize}
                    type="button"
                  >
                    RESET
                  </button>
                ) : (
                  <button
                    className="mr-1 flex h-8 items-center justify-center rounded-md border border-red-400  bg-red-400 pl-1 pr-1 text-sm text-gray-50"
                    onClick={onClickRemove}
                    type="button"
                  >
                    <Image
                      src="/assets/icons/delete.svg"
                      alt="delete.svg"
                      priority={true}
                      width={24}
                      height={24}
                      style={{ minWidth: '1.5rem', maxWidth: '1.5rem' }}
                    />
                  </button>
                )}
                <button
                  className="ml-1 mr-1 flex h-8 items-center justify-center rounded-md border border-blue-400 bg-blue-400 pl-5 pr-5 text-sm text-gray-50 disabled:border-gray-400 disabled:bg-gray-400"
                  type="submit"
                  disabled={!isValid}
                >
                  SAVE
                </button>
                <button
                  className="ml-1 flex h-8 items-center justify-center rounded-md border border-gray-600 pl-5 pr-5 text-sm text-gray-600"
                  onClick={() => onClose()}
                  type="button"
                >
                  EXIT
                </button>
              </div>
            </div>
            {/* Metric ID */}
            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text">
                  Metric ID {DEFINITION_ID_RULE_DESCRIPTION}
                </span>
                {/* <span className="label-text-alt">label-text-alt</span> */}
              </label>
              <Controller
                control={control}
                name="id"
                rules={{ required: true }}
                render={({ field: { onChange, value } }) => (
                  <input
                    type="text"
                    placeholder="Metric ID"
                    className="input-bordered input my-2 w-full px-4 text-sm focus:outline-none"
                    autoComplete="off"
                    autoCapitalize="off"
                    autoCorrect="off"
                    value={value ?? ''}
                    onChange={(event) => {
                      event.target.value = transformDefinitionId(
                        event.target.value
                      );
                      onChange(event);
                    }}
                  />
                )}
              />
            </div>
            {/* Metric Collector */}
            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text">Metric Collector</span>
              </label>
              <select
                className="select-bordered select select-sm my-2 h-12 w-full px-4 text-sm focus:outline-none"
                {...register('collector', {
                  required: true,
                  validate: (value) => !!value,
                })}
              >
                {METRIC_COLLECTOR_OPTIONS}
              </select>
            </div>
            {/* Enabled */}
            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text">Enabled</span>
              </label>
              <input
                type="checkbox"
                className="checkbox"
                {...register('enabled', {})}
              />
            </div>
            {/* Metadata */}
            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text">
                  Metric Metadata (YAML)
                </span>
                {/* <span className="label-text-alt">label-text-alt</span> */}
              </label>
              <div className="textarea-bordered textarea textarea-sm my-2 w-full px-4 py-4 focus:outline-none">
                <Controller
                  control={control}
                  name="metadata"
                  render={({ field: { onChange, value } }) => (
                    <YAMLEditor value={value} onChange={onChange} />
                  )}
                />
              </div>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
}
