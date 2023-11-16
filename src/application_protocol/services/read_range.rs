use crate::{
    application_protocol::{
        confirmed::ConfirmedServiceChoice,
        primitives::data_value::{BitString, Date, Time},
    },
    common::{
        error::{Error, Unimplemented},
        helper::{
            decode_context_object_id, decode_context_property_id, decode_signed, decode_unsigned,
            encode_application_signed, encode_application_unsigned, encode_closing_tag,
            encode_context_enumerated, encode_context_object_id, encode_context_unsigned,
            encode_opening_tag, get_tagged_body_for_tag,
        },
        io::{Reader, Writer},
        object_id::ObjectId,
        property_id::PropertyId,
        spec::BACNET_ARRAY_ALL,
        tag::{ApplicationTagNumber, Tag, TagNumber},
    },
};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadRange {
    pub object_id: ObjectId,     // e.g ObjectTrendLog
    pub property_id: PropertyId, // e.g. PropLogBuffer
    pub array_index: u32,        // use BACNET_ARRAY_ALL for all
    pub request_type: ReadRangeRequestType,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReadRangeRequestType {
    ByPosition(ReadRangeByPosition),
    BySequence(ReadRangeBySequence),
    ByTime(ReadRangeByTime),
    All,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadRangeByPosition {
    pub index: u32,
    pub count: u32,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadRangeBySequence {
    pub sequence_num: u32,
    pub count: u32,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadRangeByTime {
    pub date: Date,
    pub time: Time,
    pub count: u32,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadRangeAck<'a> {
    pub object_id: ObjectId,
    pub property_id: PropertyId,
    pub array_index: u32,
    pub result_flags: BitString<'a>,
    pub item_count: usize,
    pub item_data: ReadRangeItems<'a>,
}

impl<'a> ReadRangeAck<'a> {
    const OBJECT_ID_TAG: u8 = 0;
    const PROPERTY_ID_TAG: u8 = 1;
    const ARRAY_INDEX_TAG: u8 = 2;
    const RESULT_FLAGS_TAG: u8 = 3;
    const ITEM_COUNT_TAG: u8 = 4;
    const ITEM_DATA_TAG: u8 = 5;

    pub fn encode(&self, writer: &mut Writer) {
        writer.push(ConfirmedServiceChoice::ReadRange as u8);
        encode_context_object_id(writer, Self::OBJECT_ID_TAG, &self.object_id);
        encode_context_enumerated(writer, Self::PROPERTY_ID_TAG, &self.property_id);
        if self.array_index != BACNET_ARRAY_ALL {
            encode_context_unsigned(writer, Self::ARRAY_INDEX_TAG, self.array_index)
        }
        self.result_flags
            .encode_context(Self::RESULT_FLAGS_TAG, writer);
        encode_context_unsigned(writer, Self::ITEM_COUNT_TAG, self.item_count as u32);

        // item data
        encode_opening_tag(writer, Self::ITEM_DATA_TAG);
        self.item_data.encode(writer);
        encode_closing_tag(writer, Self::ITEM_DATA_TAG);
    }

    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Result<Self, Error> {
        // object_id
        let tag = Tag::decode_expected(
            reader,
            buf,
            TagNumber::ContextSpecific(Self::OBJECT_ID_TAG),
            "ReadRangeAck decode object_id",
        )?;
        let object_id = ObjectId::decode(tag.value, reader, buf)?;

        // property_id
        let property_id = decode_context_property_id(
            reader,
            buf,
            Self::PROPERTY_ID_TAG,
            "ReadRangeAck decode property_id",
        )?;

        // array_index
        let mut tag = Tag::decode(reader, buf)?;
        let mut array_index = BACNET_ARRAY_ALL;
        if let TagNumber::ContextSpecific(Self::ARRAY_INDEX_TAG) = tag.number {
            array_index = decode_unsigned(tag.value, reader, buf)? as u32;

            // read another tag
            tag = Tag::decode(reader, buf)?;
        }

        // result flags
        tag.expect_number(
            "ReadRangeAck decode result_flag",
            TagNumber::ContextSpecific(Self::RESULT_FLAGS_TAG),
        )?;
        let result_flags = BitString::decode(&property_id, tag.value, reader, buf)?;

        // item_count
        let tag = Tag::decode_expected(
            reader,
            buf,
            TagNumber::ContextSpecific(Self::ITEM_COUNT_TAG),
            "ReadRangeAck decode item_count",
        )?;
        let item_count = decode_unsigned(tag.value, reader, buf)? as usize;

        // item_data
        let buf = if reader.eof() {
            &[]
        } else {
            get_tagged_body_for_tag(
                reader,
                buf,
                Self::ITEM_DATA_TAG,
                "ReadRangeAck decode item_data",
            )?
        };
        let item_data = ReadRangeItems::new_from_buf(buf);

        Ok(Self {
            object_id,
            property_id,
            array_index,
            result_flags,
            item_count,
            item_data,
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadRangeItems<'a> {
    pub items: &'a [ReadRangeItem<'a>],
    reader: Reader,
    buf: &'a [u8],
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ReadRangeValue {
    Status,
    Bool(bool),
    Real(f32),
    Enum(u32),
    Unsigned(u32),
    Signed(i32),
    Bits,
    Null,
    Error,
    Delta,
    Any,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum ReadRangeValueType {
    Status = 0,
    Bool = 1,
    Real = 2,
    Enum = 3,
    Unsigned = 4,
    Signed = 5,
    Bits = 6,
    Null = 7,
    Error = 8,
    Delta = 9,
    Any = 10,
}

impl TryFrom<u8> for ReadRangeValueType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, u8> {
        match value {
            0 => Ok(Self::Status),
            1 => Ok(Self::Bool),
            2 => Ok(Self::Real),
            3 => Ok(Self::Enum),
            4 => Ok(Self::Unsigned),
            5 => Ok(Self::Signed),
            6 => Ok(Self::Bits),
            7 => Ok(Self::Null),
            8 => Ok(Self::Error),
            9 => Ok(Self::Delta),
            10 => Ok(Self::Any),
            unknown => Err(unknown),
        }
    }
}

impl<'a> ReadRangeItems<'a> {
    const DATE_TIME_TAG: u8 = 0;
    const VALUE_TAG: u8 = 1;
    const STATUS_FLAGS_TAG: u8 = 2;

    pub fn new_from_buf(buf: &'a [u8]) -> Self {
        let reader = Reader {
            index: 0,
            end: buf.len(),
        };

        Self {
            items: &[],
            reader,
            buf,
        }
    }

    pub fn new(items: &'a [ReadRangeItem<'a>]) -> Self {
        Self {
            items,
            reader: Reader::default(),
            buf: &[],
        }
    }

    pub fn encode(&self, writer: &mut Writer) {
        for item in self.items {
            // date and time
            Tag::new(TagNumber::ContextSpecificOpening(Self::DATE_TIME_TAG), 0).encode(writer);
            Tag::new(
                TagNumber::Application(ApplicationTagNumber::Date),
                Date::LEN,
            )
            .encode(writer);
            item.date.encode(writer);
            Tag::new(
                TagNumber::Application(ApplicationTagNumber::Time),
                Time::LEN,
            )
            .encode(writer);
            item.time.encode(writer);
            Tag::new(TagNumber::ContextSpecificClosing(Self::DATE_TIME_TAG), 0).encode(writer);

            // value
            Tag::new(TagNumber::ContextSpecificOpening(Self::VALUE_TAG), 0).encode(writer);
            match item.value {
                ReadRangeValue::Real(value) => {
                    Tag::new(
                        TagNumber::ContextSpecific(ReadRangeValueType::Real as u8),
                        4,
                    )
                    .encode(writer);
                    writer.extend_from_slice(&value.to_be_bytes());
                }
                _ => todo!("{:?}", item.value),
            }
            Tag::new(TagNumber::ContextSpecificClosing(Self::VALUE_TAG), 0).encode(writer);

            // status
            item.status_flags
                .encode_context(Self::STATUS_FLAGS_TAG, writer);
        }
    }

    fn next_internal(&mut self) -> Result<ReadRangeItem<'a>, Error> {
        // date and time
        Tag::decode_expected(
            &mut self.reader,
            self.buf,
            TagNumber::ContextSpecificOpening(Self::DATE_TIME_TAG),
            "ReadRangeItems next_internal",
        )?;
        Tag::decode_expected(
            &mut self.reader,
            self.buf,
            TagNumber::Application(ApplicationTagNumber::Date),
            "ReadRangeItems next_internal",
        )?;
        let date = Date::decode(&mut self.reader, self.buf)?;
        Tag::decode_expected(
            &mut self.reader,
            self.buf,
            TagNumber::Application(ApplicationTagNumber::Time),
            "ReadRangeItems next_internal",
        )?;
        let time = Time::decode(&mut self.reader, self.buf)?;
        Tag::decode_expected(
            &mut self.reader,
            self.buf,
            TagNumber::ContextSpecificClosing(Self::DATE_TIME_TAG),
            "ReadRangeItems next_internal",
        )?;

        // value
        Tag::decode_expected(
            &mut self.reader,
            self.buf,
            TagNumber::ContextSpecificOpening(Self::VALUE_TAG),
            "ReadRangeItems next_internal",
        )?;
        let tag = Tag::decode(&mut self.reader, self.buf)?;
        let value_type: ReadRangeValueType = match tag.number {
            TagNumber::ContextSpecific(tag_number) => tag_number
                .try_into()
                .map_err(|x| Error::InvalidVariant(("ReadRangeValueType", x as u32)))?,
            x => return Err(Error::TagNotSupported(("ReadRangeItems next value", x))),
        };
        let value = match value_type {
            ReadRangeValueType::Real => {
                let value = f32::from_be_bytes(self.reader.read_bytes(self.buf)?);
                ReadRangeValue::Real(value)
            }
            x => return Err(Error::Unimplemented(Unimplemented::ReadRangeValueType(x))),
        };
        Tag::decode_expected(
            &mut self.reader,
            self.buf,
            TagNumber::ContextSpecificClosing(Self::VALUE_TAG),
            "ReadRangeItems next_internal",
        )?;

        // status flags
        Tag::decode_expected(
            &mut self.reader,
            self.buf,
            TagNumber::ContextSpecific(Self::STATUS_FLAGS_TAG),
            "ReadRangeItems next_internal",
        )?;
        let status_flags = BitString::decode(
            &PropertyId::PropStatusFlags,
            tag.value,
            &mut self.reader,
            self.buf,
        )?;

        Ok(ReadRangeItem {
            date,
            time,
            value,
            status_flags,
        })
    }
}

impl<'a> Iterator for ReadRangeItems<'a> {
    type Item = Result<ReadRangeItem<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.eof() {
            return None;
        }

        let item = self.next_internal();
        Some(item)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadRangeItem<'a> {
    pub date: Date,
    pub time: Time,
    pub value: ReadRangeValue,
    pub status_flags: BitString<'a>,
}

impl ReadRange {
    const OBJECT_ID_TAG: u8 = 0;
    const PROPERTY_ID_TAG: u8 = 1;
    const ARRAY_INDEX_TAG: u8 = 2;
    const BY_POSITION_TAG: u8 = 3;
    const BY_SEQUENCE_TAG: u8 = 6;
    const BY_TIME_TAG: u8 = 7;

    pub fn new(
        object_id: ObjectId,
        property_id: PropertyId,
        request_type: ReadRangeRequestType,
    ) -> Self {
        Self {
            object_id,
            property_id,
            array_index: BACNET_ARRAY_ALL,
            request_type,
        }
    }

    pub fn decode(reader: &mut Reader, buf: &[u8]) -> Result<Self, Error> {
        let object_id = decode_context_object_id(
            reader,
            buf,
            Self::OBJECT_ID_TAG,
            "ReadRange decode object_id",
        )?;
        let property_id = decode_context_property_id(
            reader,
            buf,
            Self::PROPERTY_ID_TAG,
            "ReadRange decode property_id",
        )?;

        let mut tag = Tag::decode(reader, buf)?;
        let array_index = if tag.number == TagNumber::ContextSpecific(Self::ARRAY_INDEX_TAG) {
            let value = decode_unsigned(tag.value, reader, buf)? as u32;
            tag = Tag::decode(reader, buf)?;
            value
        } else {
            BACNET_ARRAY_ALL
        };

        let request_type = match tag.number {
            TagNumber::ContextSpecificOpening(Self::BY_POSITION_TAG) => {
                // index
                let index_tag = Tag::decode_expected(
                    reader,
                    buf,
                    TagNumber::Application(ApplicationTagNumber::UnsignedInt),
                    "ReadRange decode index",
                )?;
                let index = decode_unsigned(index_tag.value, reader, buf)? as u32;

                // count
                let count_tag = Tag::decode(reader, buf)?;
                let count = match count_tag.number {
                    TagNumber::Application(ApplicationTagNumber::UnsignedInt) => {
                        decode_unsigned(count_tag.value, reader, buf)? as u32
                    }
                    TagNumber::Application(ApplicationTagNumber::SignedInt) => {
                        let count = decode_signed(count_tag.value, reader, buf)?;
                        if count < 0 {
                            return Err(Error::InvalidValue("ReadRange count cannot be negative"));
                        }

                        count as u32
                    }
                    _ => {
                        return Err(Error::TagNotSupported((
                            "ReadRange count tag",
                            count_tag.number,
                        )))
                    }
                };

                // closing tag
                Tag::decode_expected(
                    reader,
                    buf,
                    TagNumber::ContextSpecificClosing(Self::BY_POSITION_TAG),
                    "ReadRange decode closing position",
                )?;

                ReadRangeRequestType::ByPosition(ReadRangeByPosition {
                    count: count as u32,
                    index,
                })
            }
            number => return Err(Error::TagNotSupported(("ReadRange opening tag", number))),
        };

        Ok(Self {
            array_index,
            object_id,
            property_id,
            request_type,
        })
    }

    pub fn encode(&self, writer: &mut Writer) {
        // object_id
        encode_context_object_id(writer, Self::OBJECT_ID_TAG, &self.object_id);

        // property_id
        encode_context_enumerated(writer, Self::PROPERTY_ID_TAG, &self.property_id);

        // array_index
        if self.array_index != BACNET_ARRAY_ALL {
            encode_context_unsigned(writer, Self::ARRAY_INDEX_TAG, self.array_index);
        }

        match &self.request_type {
            ReadRangeRequestType::ByPosition(x) => {
                encode_opening_tag(writer, Self::BY_POSITION_TAG);
                encode_application_unsigned(writer, x.index as u64);
                encode_application_signed(writer, x.count as i32);
                encode_closing_tag(writer, Self::BY_POSITION_TAG);
            }
            ReadRangeRequestType::BySequence(x) => {
                encode_opening_tag(writer, Self::BY_SEQUENCE_TAG);
                encode_application_unsigned(writer, x.sequence_num as u64);
                encode_application_signed(writer, x.count as i32);
                encode_closing_tag(writer, Self::BY_SEQUENCE_TAG);
            }
            ReadRangeRequestType::ByTime(x) => {
                encode_opening_tag(writer, Self::BY_TIME_TAG);
                x.date.encode(writer);
                x.time.encode(writer);
                encode_application_signed(writer, x.count as i32);
                encode_closing_tag(writer, Self::BY_TIME_TAG);
            }
            ReadRangeRequestType::All => {
                // do nothing
            }
        }
    }
}
