import Select, { createFilter } from 'react-select';
import React from 'react';
import { useLoader } from './loading';
import data from '../data';
import Modal from 'react-modal';

// The minimum number of characters entered before showing states.
const COUNTRY_THRESHOLD = 3;

const POPUP_STYLE = {
    marginTop: '64px',
};

export default function RegionSelect({ showing, region, setRegion }) {
    let [err1, regionList] = useLoader(data.region.list, []);
    let [changing, setChanging] = React.useState(false);

    let regions = React.useMemo(() => {
        if (regionList == null) return null;
        let result = {};
        for (let x of regionList) result[x.id] = x;
        return result;
    }, [regionList]);

    let countries = React.useMemo(() => {
        if (regionList == null) return null;
        let result = {}
        for (let x of regionList) if (!x.id.includes('-')) result[x.id] = x;
        return result;
    }, [regionList]);

    if (err1) return <div>An error occurred.</div>;
    if (regions == null) return <div>Loading…</div>;

    let name = regions.hasOwnProperty(region) ? regions[region].name : region;

    let onSelectChange = value => {
        setChanging(false);
        setRegion(value.id);
    };

    let selectOptions = regionList;
    let value = regions.hasOwnProperty(region) ? regions[region] : null;

    let getOptionLabel = option => {
        let pre = '';

        if (option.id.includes('-')) {
            let country = option.id.split('-')[0];
            if (regions.hasOwnProperty(country)) {
                pre += regions[country].name + ' › ';
            }
        }

        return pre + option.name;
    };

    let getOptionValue = option => {
        return option.id;
    };

    let defaultFilter = createFilter({ ignoreAccents: false });

    let filterOption = (candidate, input) => {
        if (input.length < COUNTRY_THRESHOLD && !countries.hasOwnProperty(candidate.value)) {
            return false;
        }
        return defaultFilter(candidate, input);
    };

    return (
        <div>
            Showing {showing} for <strong>{name}</strong>. <a onClick={() => setChanging(true)}>Change Region</a>
            <Modal
                className='reading-width'
                style={{ content: POPUP_STYLE }}
                isOpen={changing}
                onRequestClose={() => setChanging(false)}
                shouldCloseOnOverlayClick={true}
                shouldCloseOnEsc={true}
                shouldReturnFocusAfterClose={true}
            >
                <Select
                    autoFocus
                    backspaceRemovesValue={false}
                    controlShouldRenderValue={false}
                    components={{ DropdownIndicator: null, IndicatorSeparator: null }}
                    filterOption={filterOption}
                    hideSelectedOptions={false}
                    isClearable={false}
                    menuIsOpen={true}
                    onChange={onSelectChange}
                    options={selectOptions}
                    placeholder='Region Name…'
                    tabSelectsValue={false}
                    getOptionLabel={getOptionLabel}
                    getOptionValue={getOptionValue}
                    noOptionsMessage={input => 'No regions found.'}
                />
                <p>Modal Content</p>
            </Modal>
        </div>
    );
}
