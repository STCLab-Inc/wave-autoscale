'use client';

import React, { useEffect, useMemo, useState } from 'react';
import { useRouter } from 'next/navigation';
import { useForm } from 'react-hook-form';
import { forEach } from 'lodash';

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

  return (
    <div className="metric-drawer drawer drawer-end absolute bottom-0 top-0 z-50 border-l border-t border-gray-200">
      <input id="drawer" type="checkbox" className="drawer-toggle" checked />
      <div className="drawer-side">
        <label
          htmlFor="drawer"
          className="drawer-overlay"
          onClick={onClickOverlay}
        />
        <div className="drawer-content min-w-[32rem] overflow-y-auto bg-base-100 p-4">
          <form className="" onSubmit={handleSubmit(onSubmit)}>
            <div className="mb-4 flex items-center justify-between">
              <h2 className="font-bold">Metric</h2>
              <div>
                {isNew ? undefined : (
                  <button
                    type="button"
                    className="btn-error btn-sm btn mr-2"
                    onClick={onClickRemove}
                  >
                    Remove
                  </button>
                )}

                <button type="submit" className="btn-primary btn-sm btn">
                  Save
                </button>
              </div>
            </div>
            <div className="form-control mb-4 w-full">
              <label className="label">
                <span className="label-text">Metric Type</span>
                <span className="label-text-alt"></span>
              </label>
              <select
                className="select-bordered select"
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

            {metadataFormControls}
          </form>
        </div>
      </div>
    </div>
  );
}
