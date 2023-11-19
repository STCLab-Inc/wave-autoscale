'use client';

import React, { useEffect, useMemo, useState } from 'react';
import { useRouter } from 'next/navigation';
import { useForm } from 'react-hook-form';
import { forEach } from 'lodash';
import Image from 'next/image';

import { getMetadataFormControls } from './metadata-form-controls';
import {
  generateMetricDefinition,
  getMetricKeyTypes,
} from '@/utils/metric-binding';
import MetricService from '@/services/metric';
import { MetricDefinition } from '@/types/bindings/metric-definition';

// Metric Types
const metricKeyTypes = getMetricKeyTypes();
const metricOptions = metricKeyTypes.map((metricKeyType) => (
  <option key={metricKeyType.metricName} value={metricKeyType.metricName}>
    {metricKeyType.metricName}
  </option>
));

export default function MetricDetailDrawer({
  metricDefinition,
}: {
  metricDefinition?: MetricDefinition;
}) {
  const {
    register,
    handleSubmit,
    watch,
    setValue,
    getValues,
    reset,
    formState: { errors },
  } = useForm();
  const router = useRouter();
  const dbId = metricDefinition?.db_id;
  const isNew = !dbId;
  const [selectedMetricType, setSelectedMetricType] = useState<string>();

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

  const goBack = (refresh?: boolean) => {
    let path = window.location.href;
    path = path.slice(0, path.lastIndexOf('/'));
    router.push(path);
    if (refresh) {
      router.refresh();
    }
  };

  //
  // Events
  //
  useEffect(() => {
    if (isNew) {
      return;
    }
    const { id, metric_kind, metadata, ...rest } = metricDefinition;
    setValue('id', id);
    setValue('metric_kind', metric_kind);
    forEach(metadata, (value, key) => {
      setValue(key, value);
    });
    setSelectedMetricType(metric_kind);
  }, [metricDefinition, isNew]);

  const onClickOverlay = () => {
    goBack(false);
  };

  const onChangeMetricType = (e: any) => {
    const metricType = e.target.value;
    setSelectedMetricType(metricType);

    // Clear metadata
    const id = getValues('id');
    reset();
    setValue('id', id);
    setValue('metric_kind', metricType);
  };

  // Exit metrics drawer
  const onClickExit = async () => {
    goBack();
  };

  const onClickInitialize = async () => {
    setSelectedMetricType('Metric Kind');

    // Clear metadata
    const id = getValues('id');
    reset();
    setValue('id', id);
    setValue('metric_kind', 'Metric Kind');
  };

  const onClickRemove = async () => {
    if (isNew || !dbId) {
      return;
    }
    try {
      await MetricService.deleteMetric(dbId);
      goBack(true);
    } catch (error) {
      console.log(error);
    }
  };

  const onSubmit = async (data: any) => {
    const { id, metric_kind, ...metadata } = data;
    const metricDefinition = generateMetricDefinition({
      id,
      db_id: isNew ? '' : dbId,
      metric_kind,
      metadata,
    });
    try {
      if (isNew) {
        const result = await MetricService.createMetric(metricDefinition);
        console.log({ result, isNew });
      } else {
        const result = await MetricService.updateMetric(metricDefinition);
        console.log({ result });
      }
      goBack(true);
    } catch (error) {
      console.log(error);
    }
  };

  console.log(metadataFormControls);

  return (
    <div className="metric-drawer drawer drawer-end fixed bottom-0 right-0 top-16 z-50 w-full border-t border-gray-200">
      <input id="drawer" type="checkbox" className="drawer-toggle" checked />
      <div className="drawer-side h-full border-t border-gray-200">
        <label
          htmlFor="drawer"
          className="drawer-overlay"
          onClick={onClickOverlay}
        />
        <div className="drawer-content flex h-full min-w-[25rem] flex-col overflow-y-auto border-l border-gray-200 bg-base-100 pb-20">
          <form className="" onSubmit={handleSubmit(onSubmit)}>
            <div className="flex h-14 min-h-14 w-full min-w-full flex-row items-center justify-between border-b border-dashed border-gray-400 bg-gray-75">
              <span className="font-Pretendard truncate whitespace-nowrap px-4 text-lg font-semibold text-gray-1000">
                Metric
              </span>
              <div className="flex px-4">
                {isNew ? (
                  <button
                    className="mr-1 flex h-8 items-center justify-center rounded-md border border-red-400  bg-red-400 pl-1 pr-1 text-sm text-gray-50"
                    onClick={onClickInitialize}
                  >
                    <Image
                      src="/assets/icons/initialize.svg"
                      alt="initialize.svg"
                      priority={true}
                      width={24}
                      height={24}
                      style={{ minWidth: '1.5rem', maxWidth: '1.5rem' }}
                    />
                  </button>
                ) : (
                  <button
                    className="mr-1 flex h-8 items-center justify-center rounded-md border border-red-400  bg-red-400 pl-1 pr-1 text-sm text-gray-50"
                    onClick={onClickRemove}
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
                  className="ml-1 mr-1 flex h-8 items-center justify-center rounded-md border border-blue-400 bg-blue-400 pl-5 pr-5 text-sm text-gray-50"
                  type="submit"
                >
                  SAVE
                </button>
                <button
                  className="ml-1 flex h-8 items-center justify-center rounded-md border border-gray-600 pl-5 pr-5 text-sm text-gray-600"
                  onClick={onClickExit}
                >
                  EXIT
                </button>
              </div>
            </div>

            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text px-2">Metric Type</span>
                {/* <span className="label-text-alt">label-text-alt</span> */}
              </label>
              <select
                className="select-bordered select my-2 flex w-full truncate rounded-md text-sm focus:outline-none"
                defaultValue="Metric Kind"
                {...register('metric_kind', {
                  required: true,
                  onChange: onChangeMetricType,
                })}
              >
                <option disabled>Metric Kind</option>
                {metricOptions}
              </select>
            </div>

            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text px-2">Metric ID</span>
                {/* <span className="label-text-alt">label-text-alt</span> */}
              </label>
              <input
                type="text"
                placeholder="Metric ID"
                className="input-bordered input my-2 w-full px-4 text-sm focus:outline-none"
                autoComplete="off"
                autoCapitalize="off"
                autoCorrect="off"
                {...register('id', { required: true })}
              />
            </div>

            {metadataFormControls}
          </form>
        </div>
      </div>
    </div>
  );
}
