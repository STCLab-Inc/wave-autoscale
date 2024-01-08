'use client';

import React, { useEffect, useMemo, useState } from 'react';
import { useSearchParams, useRouter } from 'next/navigation';

import dayjs, { Dayjs } from 'dayjs';

import InflowService from '@/services/inflow';
import { InflowDefinition } from '@/types/bindings/inflow-definition';

import InflowDetailDrawer from './inflow-drawer';
import ContentHeader from '../common/content-header';
import { TableComponent } from '../common/table';

const formatDate = (date: Dayjs) => date.format('YYYY-MM-DD');

async function getInflowMetricId() {
  const metric_ids = await InflowService.getInflowMetricId();
  return metric_ids;
}
async function getInflowWithMetricIdByDate(
  metric_id: string,
  from: Dayjs,
  to: Dayjs
) {
  const inflow = await InflowService.getInflowWithMetricIdByDate(
    metric_id,
    from,
    to
  );
  return inflow;
}

const DEFAULT_FROM = dayjs().subtract(7, 'days');
const DEFAULT_TO = dayjs();

interface InflowDefinitionEx extends InflowDefinition {
  isChecked: boolean;
}

export default function InflowPage() {
  const router = useRouter();
  const searchParams = useSearchParams();

  const fromParam = searchParams.get('from');
  const toParam = searchParams.get('to');
  const from = fromParam || formatDate(DEFAULT_FROM);
  const to = toParam || formatDate(DEFAULT_TO);
  const fromDayjs = useMemo(() => dayjs(from).startOf('day'), [from]);
  const toDayjs = useMemo(() => dayjs(to).endOf('day'), [to]);

  const pageParam = searchParams.get('page');
  const page = pageParam ? parseInt(pageParam, 10) : 1;
  const sizeParam = searchParams.get('size');
  const size = sizeParam ? parseInt(sizeParam, 10) : 10;

  const [inflow, setInflow] = useState<InflowDefinitionEx[]>([]);
  const [inflowItem, setInflowItem] = useState<InflowDefinitionEx>();

  const [metricIds, setMetricIds] = useState<string[]>(['Metric ID']);
  const [metricId, setMetricId] = useState<string>('');

  const fetchInflow = async () => {
    try {
      if (!metricId) {
        let metricIdsData = await getInflowMetricId();
        setMetricIds([...metricIds, ...metricIdsData]);
      } else if (metricId) {
        let inflowData = await getInflowWithMetricIdByDate(
          metricId,
          fromDayjs,
          toDayjs
        );
        setInflow(inflowData);
      }
    } catch (error) {
      console.error({ error });
      return [];
    }
  };

  useEffect(() => {
    fetchInflow();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [fromDayjs, toDayjs, metricId]);

  const handleDate = (field: 'from' | 'to', value: string) => {
    const newDate = dayjs(value);
    const isValidDate = newDate.isValid();

    if (isValidDate) {
      const params = {
        from: field === 'from' ? formatDate(newDate) : from,
        to: field === 'to' ? formatDate(newDate) : to,
      };
      router.push(
        `/app/inflow?from=${params.from}&to=${
          params.to
        }&page=${1}&size=${sizePerPage}`
      );
    }
  };

  const [selectAll, setSelectAll] = useState(false);

  const handleSelectAll = (event: React.ChangeEvent<HTMLInputElement>) => {
    const checked = event.target.checked;
    setSelectAll(checked);
    const updatedInflowData = inflow.map((updatedInflowDataItem) => ({
      ...updatedInflowDataItem,
      isChecked: checked,
    }));
    setInflow(updatedInflowData);
  };

  const SIZE_PER_PAGE_OPTIONS = [10, 50, 100, 200, 500];

  const [sizePerPage, setSizePerPage] = useState(SIZE_PER_PAGE_OPTIONS[0]);
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPage, setTotalPage] = useState(1);

  useEffect(() => {
    setTotalPage(Math.ceil(inflow.length / sizePerPage) || 1);
  }, [inflow, from, to, currentPage, totalPage, sizePerPage]);

  const handleSizePerPage = (event: React.ChangeEvent<HTMLSelectElement>) => {
    const newItemsPerPage = parseInt(event.target.value, 10);
    setSizePerPage(newItemsPerPage);
    setCurrentPage(1);
  };

  const handleCurrentPage = (newPage: number) => {
    setCurrentPage(newPage);
  };

  useEffect(() => {
    setCurrentPage(page || 1);
    setSizePerPage(size || SIZE_PER_PAGE_OPTIONS[0]);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [page, size]);

  useEffect(() => {
    if (currentPage > totalPage || currentPage < 1) {
      setCurrentPage(1);
    }
    router.push(
      `/app/inflow?from=${from}&to=${to}&page=${currentPage}&size=${sizePerPage}`
    );
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [inflow, from, to, currentPage, totalPage, sizePerPage, searchParams]);

  const [detailsModalFlag, setDetailsModalFlag] = useState(false);

  const onClickDetails = (InflowItem: InflowDefinitionEx | undefined) => {
    setInflowItem(InflowItem);
    setDetailsModalFlag(true);
  };

  const [fetchFlag, setFetchFlag] = useState(false);

  useEffect(() => {
    if (fetchFlag) {
      fetchInflow();
      setFetchFlag(false);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [fetchFlag]);

  const handleFieldChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    setMetricId(event.target.value);
  };

  const tableFormat = [
    {
      label: 'checkbox',
      type: 'checkbox',
      content: 'isChecked',
      weight: '1',
      status: selectAll,
      function: handleSelectAll,
    },
    {
      label: 'JSON Value',
      type: 'span',
      content: 'json_value',
      format: 'json',
      weight: '10',
    },
    {
      label: 'Actions',
      type: 'button',
      content: 'dataItem',
      format: 'click',
      weight: '1',
      function: onClickDetails,
    },
  ];

  return (
    <main className="flex h-full w-full flex-row">
      <div className="flex h-full w-full flex-col">
        <div className="flex h-full w-full flex-col">
          <ContentHeader
            type="OUTER"
            title="Metrics Inflow"
            right={
              <div className="flex items-center">
                <div className="mx-4 flex h-8 items-center">
                  <div className="mx-2 flex items-center">
                    <label className="select-group-sm">
                      <select
                        value={metricId}
                        onChange={handleFieldChange}
                        className="focus:outline-noneselect select-sm min-w-[250px] cursor-pointer rounded-md border border-gray-200 px-2"
                      >
                        {metricIds.map((item, index) => (
                          <option key={index} value={item}>
                            {item}
                          </option>
                        ))}
                      </select>
                    </label>
                  </div>
                </div>
                <div className="form-control mr-2">
                  <label className="input-group-sm">
                    <input
                      type="date"
                      className="input-bordered input input-sm max-w-[130px] cursor-text px-2 text-center focus:outline-none"
                      max={formatDate(toDayjs)}
                      value={from}
                      onChange={(event) =>
                        handleDate('from', event.target.value)
                      }
                    />
                  </label>
                </div>
                <span>-</span>
                <div className="form-control ml-2">
                  <label className="input-group-sm">
                    <input
                      type="date"
                      className="input-bordered input input-sm max-w-[130px] cursor-text px-2 text-center focus:outline-none"
                      min={formatDate(fromDayjs)}
                      value={to}
                      onChange={(event) => handleDate('to', event.target.value)}
                    />
                  </label>
                </div>
              </div>
            }
          />
          <div className="flex w-full flex-col">
            <TableComponent
              tableFormat={tableFormat}
              /*  */
              data={inflow}
              /*  */
              sizePerPageOptions={SIZE_PER_PAGE_OPTIONS}
              sizePerPage={sizePerPage}
              handleSizePerPage={handleSizePerPage}
              /*  */
              currentPage={currentPage}
              totalPage={totalPage}
              handleCurrentPage={handleCurrentPage}
            />
          </div>
        </div>
        {detailsModalFlag ? (
          <InflowDetailDrawer
            inflowItem={inflowItem}
            setDetailsModalFlag={setDetailsModalFlag}
            setFetchFlag={setFetchFlag}
          />
        ) : null}
      </div>
    </main>
  );
}
