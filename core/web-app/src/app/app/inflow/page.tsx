'use client';

import React, { useEffect, useMemo, useState } from 'react';
import { useSearchParams, useRouter } from 'next/navigation';

import dayjs, { Dayjs } from 'dayjs';
import { decodeTime } from 'ulid';

import InflowService from '@/services/inflow';
import { InflowDefinition } from '@/types/bindings/inflow-definition';

import ContentHeader from '../content-header';
/* import HistoryHeatmap from './history-heatmap'; */
import { renderKeyValuePairsWithJson } from '../keyvalue-renderer';
import InflowDetailDrawer from './inflow-drawer';

const formatDate = (date: Dayjs) => date.format('YYYY-MM-DD');

async function getInflow(from: Dayjs, to: Dayjs) {
  const inflow = await InflowService.getInflowByFromTo(from, to);
  return inflow;
}

const DEFAULT_FROM = dayjs().subtract(7, 'days');
const DEFAULT_TO = dayjs();

interface InflowDefinitionEx extends InflowDefinition {
  created_at: string;
  isChecked: boolean;
}

export default function InflowPage() {
  const router = useRouter();
  const searchParams = useSearchParams();

  const fromParam = searchParams.get('from');
  const toParam = searchParams.get('to');
  const from = fromParam || formatDate(DEFAULT_FROM);
  const to = toParam || formatDate(DEFAULT_TO);
  const fromDayjs = useMemo(() => dayjs(from), [from]);
  const toDayjs = useMemo(() => dayjs(to).endOf('day'), [to]);

  const pageParam = searchParams.get('page');
  const page = pageParam ? parseInt(pageParam, 10) : 1;
  const viewParam = searchParams.get('view');
  const view = viewParam ? parseInt(viewParam, 10) : 10;

  const [inflow, setInflow] = useState<InflowDefinitionEx[]>([]);
  const [inflowItem, setInflowItem] = useState<InflowDefinitionEx>();

  const fetchInflow = async () => {
    try {
      const id = inflowItem?.id;
      let inflow = await getInflow(fromDayjs, toDayjs);
      inflow = inflow.map((InflowItem: InflowDefinition) => ({
        ...InflowItem,
        created_at: decodeTime(InflowItem.id),
      }));
      setInflow(inflow);
      setInflowItem(inflow.find((item: InflowDefinition) => item.id === id));
    } catch (error) {
      console.error({ error });
      return [];
    }
  };

  useEffect(() => {
    fetchInflow();
  }, [fromDayjs, toDayjs]);

  const handleDateChange = (field: 'from' | 'to', value: string) => {
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
        }&page=${1}&view=${itemsPerPage}`
      );
    }
  };

  const [checkAllFlag, setCheckAllFlag] = useState(false);

  const handleCheckAllChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const checked = event.target.checked;
    setCheckAllFlag(checked);
    const updatedInflowItem = inflow.map((InflowItem) => ({
      ...InflowItem,
      isChecked: checked,
    }));
    setInflow(updatedInflowItem);
  };

  const ITEMS_PER_PAGE_OPTIONS = [10, 50, 100, 200, 500];

  const [itemsPerPage, setItemsPerPage] = useState(ITEMS_PER_PAGE_OPTIONS[0]);
  const [currentPage, setCurrentPage] = useState(1);

  const totalPageCount = Math.ceil(inflow.length / itemsPerPage) || 1;

  const startIndex = (currentPage - 1) * itemsPerPage;
  const endIndex = startIndex + itemsPerPage;
  const visibleInflow = inflow.slice(startIndex, endIndex);

  const handleItemsPerPageChange = (
    event: React.ChangeEvent<HTMLSelectElement>
  ) => {
    const newItemsPerPage = parseInt(event.target.value, 10);
    setItemsPerPage(newItemsPerPage);
    setCurrentPage(1);
  };

  const handlePageChange = (newPage: number) => {
    setCurrentPage(newPage);
  };

  useEffect(() => {
    setCurrentPage(page ? page : 1);
    setItemsPerPage(view ? view : ITEMS_PER_PAGE_OPTIONS[0]);
  }, [page, view]);

  useEffect(() => {
    if (currentPage > totalPageCount) {
      setCurrentPage(1);
    }
    router.push(
      `/app/inflow?from=${from}&to=${to}&page=${currentPage}&view=${itemsPerPage}`
    );
  }, [currentPage, itemsPerPage, fromParam, toParam, pageParam, viewParam]);

  const [detailsModalFlag, setDetailsModalFlag] = useState(false);

  const onClickDetails = (InflowItem: InflowDefinitionEx) => {
    setInflowItem(InflowItem);
    setDetailsModalFlag(true);
  };

  const [fetchFlag, setFetchFlag] = useState(false);

  useEffect(() => {
    if (fetchFlag) {
      fetchInflow();
      setFetchFlag(false);
    }
  }, [fetchFlag]);

  return (
    <main className="flex h-full w-full flex-row">
      <div className="flex h-full w-full flex-col">
        <div className="flex h-full w-full flex-col">
          <ContentHeader
            title="Inflow"
            right={
              <div className="flex items-center">
                <div className="form-control mr-2">
                  <label className="input-group-sm">
                    <input
                      type="date"
                      className="input-bordered input input-sm max-w-[130px] cursor-text px-2 text-center focus:outline-none"
                      max={formatDate(toDayjs)}
                      value={from}
                      onChange={(event) =>
                        handleDateChange('from', event.target.value)
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
                      onChange={(event) =>
                        handleDateChange('to', event.target.value)
                      }
                    />
                  </label>
                </div>
              </div>
            }
          />
          <div className="flex w-full flex-col">
            {/* <HistoryHeatmap inflow={inflow} from={fromDayjs} to={toDayjs} /> */}
            <div className="flex items-center justify-end px-8 py-4">
              <div className="mr-2 flex items-center">
                <label className="select-group-sm">
                  <select
                    value={itemsPerPage}
                    onChange={handleItemsPerPageChange}
                    className="focus:outline-noneselect select-sm max-w-[130px] cursor-pointer rounded-md border border-gray-200 px-2"
                  >
                    {ITEMS_PER_PAGE_OPTIONS.map((option) => (
                      <option key={option} value={option}>
                        {option}
                      </option>
                    ))}
                  </select>
                </label>
              </div>

              <div className="mx-2 flex items-center justify-center">
                <span className="px-2 text-center text-sm">
                  {currentPage} / {totalPageCount}
                </span>
              </div>

              <div className="ml-2 flex h-8 items-center">
                <button
                  className={
                    currentPage === 1
                      ? 'mr-1 flex h-8 cursor-not-allowed items-center justify-center rounded-md border border-gray-400 bg-gray-400 pl-5 pr-5 text-sm text-gray-50'
                      : 'mr-1 flex h-8 cursor-pointer items-center justify-center rounded-md border border-red-400 bg-red-400 pl-5 pr-5 text-sm text-gray-50'
                  }
                  onClick={() => handlePageChange(currentPage - 1)}
                  disabled={currentPage === 1}
                >
                  PREVIOUS
                </button>
                <button
                  className={
                    currentPage &&
                    totalPageCount &&
                    currentPage !== totalPageCount
                      ? 'ml-1 flex h-8 cursor-pointer items-center justify-center rounded-md border border-blue-400 bg-blue-400 pl-5 pr-5 text-sm text-gray-50'
                      : 'ml-1 flex h-8 cursor-not-allowed items-center justify-center rounded-md border border-gray-400 bg-gray-400 pl-5 pr-5 text-sm text-gray-50'
                  }
                  onClick={() => handlePageChange(currentPage + 1)}
                  disabled={currentPage === totalPageCount}
                >
                  NEXT
                </button>
              </div>
            </div>

            <table className="flex w-full flex-col">
              <thead className="text-md flex h-12 w-full items-center justify-between border-b border-t bg-gray-75 py-0 font-bold text-gray-800">
                <tr className="flex h-full w-full px-8">
                  <th className="mr-4 flex h-full flex-1 items-center">
                    <label className="flex h-full items-center">
                      <input
                        type="checkbox"
                        className="checkbox"
                        checked={checkAllFlag}
                        onChange={handleCheckAllChange}
                      />
                    </label>
                  </th>
                  <th className="mx-4 flex h-full w-full flex-4 items-center">
                    <span className="flex items-center break-words">
                      Inflow ID
                    </span>
                  </th>
                  <th className="mx-4 flex h-full w-full flex-2 items-center">
                    <span className="flex items-center break-words">
                      Collector
                    </span>
                  </th>
                  <th className="mx-4 flex h-full w-full flex-4 items-center">
                    <span className="flex items-center break-words">
                      Metric ID
                    </span>
                  </th>
                  <th className="mx-4 flex h-full w-full flex-10 items-center">
                    <span className="flex items-center break-words">
                      JSON Value
                    </span>
                  </th>
                  <th className="mx-4 flex h-full w-full flex-2 items-center">
                    <div className="flex items-center break-words">Date</div>
                  </th>
                  <th className="mx-4 flex h-full w-full flex-1 items-center">
                    <div className="flex items-center break-words">Actions</div>
                  </th>
                </tr>
              </thead>
              <tbody className="text-md min-h-12 flex w-full flex-col items-center justify-between py-0 text-gray-800">
                {visibleInflow.map((InflowItem: InflowDefinitionEx) => (
                  <tr
                    key={InflowItem.id}
                    className="flex w-full border-b px-8 py-4"
                  >
                    <td className="mr-4 flex h-full flex-1 items-start">
                      <label className="flex items-center">
                        <input
                          type="checkbox"
                          className="checkbox"
                          checked={InflowItem.isChecked}
                          onChange={(event) => {
                            const checked = event.target.checked;
                            const updatedInflowItem = inflow.map((item) =>
                              item.id === InflowItem.id
                                ? { ...item, isChecked: checked }
                                : item
                            );
                            setInflow(updatedInflowItem);
                          }}
                        />
                      </label>
                    </td>
                    <td className="mx-4 flex h-full w-full flex-4 items-start">
                      <div className="flex items-center break-all">
                        {InflowItem.id}
                      </div>
                    </td>
                    <td className="mx-4 flex h-full w-full flex-2 items-start">
                      <div className="flex items-center break-all">
                        {InflowItem.collector}
                      </div>
                    </td>
                    <td className="mx-4 flex h-full w-full flex-4 items-start">
                      <div className="flex items-center break-all">
                        {InflowItem.metric_id}
                      </div>
                    </td>
                    <td className="mx-4 flex h-full w-full flex-10 items-start">
                      <div className="flex flex-col items-center">
                        {renderKeyValuePairsWithJson(
                          InflowItem.json_value,
                          true
                        )}
                      </div>
                    </td>
                    <td className="mx-4 flex h-full w-full flex-2 items-start">
                      <div className="flex items-center break-all">
                        {dayjs
                          .unix(Number(InflowItem.created_at) / 1000)
                          .format('YYYY/MM/DD HH:mm:ss')}
                      </div>
                    </td>
                    <td className="mx-4 flex h-full w-full flex-1 items-start">
                      <div className="flex items-center">
                        <button
                          className="badge-info badge bg-[#99E3D0] px-2 py-3 text-white"
                          onClick={() => onClickDetails(InflowItem)}
                        >
                          Details
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
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
