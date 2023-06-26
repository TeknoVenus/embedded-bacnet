use alloc::{string::String, vec::Vec};

use crate::{
    application_protocol::primitives::data_value::ApplicationDataValue,
    common::{
        helper::{
            decode_unsigned, encode_closing_tag, encode_context_enumerated,
            encode_context_object_id, encode_context_unsigned, encode_opening_tag, Buffer, Reader,
        },
        object_id::ObjectId,
        property_id::PropertyId,
        spec::BACNET_ARRAY_ALL,
        tag::{Tag, TagNumber},
    },
};

#[derive(Debug)]
pub struct ReadPropertyMultipleAck {
    pub objects: Vec<ObjectWithResults>,
}

#[derive(Debug)]
pub struct ObjectWithResults {
    pub object_id: ObjectId,
    pub results: Vec<PropertyResult>,
}

#[derive(Debug)]
pub struct PropertyResult {
    pub id: PropertyId,
    pub value: PropertyValue,
}

#[derive(Debug)]
pub enum PropertyValue {
    PropValue(ApplicationDataValue),
    PropDescription(String),
    PropObjectName(String),
}

impl ReadPropertyMultipleAck {
    pub fn decode(reader: &mut Reader) -> Self {
        let mut objects = Vec::new();

        while !reader.eof() {
            let tag = Tag::decode(reader);
            assert_eq!(
                tag.number,
                TagNumber::ContextSpecific(0),
                "expected object_id tag"
            );
            let object_id = ObjectId::decode(reader, tag.value).unwrap();

            //let object_id = decode_context_object_id(reader);
            let tag = Tag::decode(reader);
            assert_eq!(
                tag.number,
                TagNumber::ContextSpecific(1),
                "expected list of results opening tag"
            );

            let mut results = Vec::new();

            loop {
                let tag = Tag::decode(reader);
                if tag.number == TagNumber::ContextSpecific(1) {
                    // closing tag
                    break;
                }

                assert_eq!(
                    tag.number,
                    TagNumber::ContextSpecific(2),
                    "expected property identifier tag"
                );
                let property_id: PropertyId = (decode_unsigned(reader, tag.value) as u32).into();

                let tag = Tag::decode(reader);
                assert_eq!(
                    tag.number,
                    TagNumber::ContextSpecific(4),
                    "expected opening tag"
                );

                let property_value = if property_id == PropertyId::PropEventTimeStamps {
                    // hack to read past complicated timestamps
                    loop {
                        let byte = reader.read_byte();
                        // read until we get to the closing tag
                        if byte == 0x4f {
                            break PropertyValue::PropValue(ApplicationDataValue::Boolean(false));
                        }
                    }
                } else {
                    let tag = Tag::decode(reader);
                    let value = ApplicationDataValue::decode(&tag, reader);
                    let property_value = PropertyValue::PropValue(value);

                    let tag = Tag::decode(reader);
                    assert_eq!(
                        tag.number,
                        TagNumber::ContextSpecific(4),
                        "expected closing tag"
                    );

                    property_value
                };

                let property_result = PropertyResult {
                    id: property_id,
                    value: property_value,
                };

                results.push(property_result)
            }

            let object_with_results = ObjectWithResults { object_id, results };
            objects.push(object_with_results);
        }

        Self { objects }
    }
}

#[derive(Debug)]
pub struct ReadPropertyMultiple {
    pub object_id: ObjectId, // e.g ObjectDevice:20088
    pub property_ids: Vec<PropertyId>,
    pub array_index: u32, // use BACNET_ARRAY_ALL for all
}

impl ReadPropertyMultiple {
    pub fn new(object_id: ObjectId, property_ids: Vec<PropertyId>) -> Self {
        Self {
            object_id,
            property_ids,
            array_index: BACNET_ARRAY_ALL,
        }
    }

    pub fn encode(&self, buffer: &mut Buffer) {
        // object_id
        encode_context_object_id(buffer, 0, &self.object_id);

        encode_opening_tag(buffer, 1);

        for property_id in &self.property_ids {
            // property_id
            encode_context_enumerated(buffer, 0, *property_id);

            // array_index
            if self.array_index != BACNET_ARRAY_ALL {
                encode_context_unsigned(buffer, 1, self.array_index);
            }
        }

        encode_closing_tag(buffer, 1);
    }

    pub fn decode(_reader: &mut Reader) -> Self {
        unimplemented!()
    }
}