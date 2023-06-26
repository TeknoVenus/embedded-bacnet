use crate::common::helper::{Buffer, Reader};

use super::{
    i_am::IAm,
    read_property::{ReadProperty, ReadPropertyAck},
    read_property_multiple::{ReadPropertyMultiple, ReadPropertyMultipleAck},
    who_is::WhoIs,
};

// Application Layer Protocol Data Unit
#[derive(Debug)]
pub enum ApplicationPdu {
    ConfirmedRequest(ConfirmedRequest),
    UnconfirmedRequest(UnconfirmedRequest),
    ComplexAck(ComplexAck),
    // add more here
}

#[repr(u8)]
pub enum ApduType {
    ConfirmedServiceRequest = 0,
    UnconfirmedServiceRequest = 1,
    SimpleAck = 2,
    ComplexAck = 3,
    SegmentAck = 4,
    Error = 5,
    Reject = 6,
    Abort = 7,
}

impl From<u8> for ApduType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::ConfirmedServiceRequest,
            1 => Self::UnconfirmedServiceRequest,
            2 => Self::SimpleAck,
            3 => Self::ComplexAck,
            4 => Self::SegmentAck,
            5 => Self::Error,
            6 => Self::Reject,
            7 => Self::Abort,
            _ => panic!("invalid pdu type"),
        }
    }
}

// preshifted by 4 bits
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum MaxSegments {
    _0 = 0x00,
    _2 = 0x10,
    _4 = 0x20,
    _8 = 0x30,
    _16 = 0x40,
    _32 = 0x50,
    _64 = 0x60,
    _65 = 0x70, // default
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum MaxAdpu {
    _0 = 0x00,
    _128 = 0x01,
    _206 = 0x02,
    _480 = 0x03,
    _1024 = 0x04,
    _1476 = 0x05, // default
}

#[derive(Debug)]
#[repr(u8)]
pub enum ConfirmedServiceChoice {
    // alarm and event services
    AcknowledgeAlarm = 0,
    AuditNotification = 32,
    CovNotification = 1,
    CovNotificationMultiple = 31,
    EventNotification = 2,
    GetAlarmSummary = 3,
    GetEnrollmentSummary = 4,
    GetEventInformation = 29,
    LifeSafetyOperation = 27,
    SubscribeCov = 5,
    SubscribeCovProperty = 28,
    SubscribeCovPropertyMultiple = 30,

    // file access services
    AtomicReadFile = 6,
    AtomicWriteFile = 7,

    // object access services
    AddListElement = 8,
    RemoveListElement = 9,
    CreateObject = 10,
    DeleteObject = 11,
    ReadProperty = 12,
    ReadPropConditional = 13,
    ReadPropMultiple = 14,
    ReadRange = 26,
    WriteProperty = 15,
    WritePropMultiple = 16,
    AuditLogQuery = 33,

    // remote device management services
    DeviceCommunicationControl = 17,
    PrivateTransfer = 18,
    TextMessage = 19,
    ReinitializeDevice = 20,

    // virtual terminal services
    VtOpen = 21,
    VtClose = 22,
    VtData = 23,

    // security services
    Authenticate = 24,
    RequestKey = 25,

    // services added after 1995
    // readRange [26] see Object Access Services
    // lifeSafetyOperation [27] see Alarm and Event Services
    // subscribeCOVProperty [28] see Alarm and Event Services
    // getEventInformation [29] see Alarm and Event Services

    // services added after 2012
    // subscribe-cov-property-multiple [30] see Alarm and Event Services
    // confirmed-cov-notification-multiple [31] see Alarm and Event Services

    // services added after 2016
    // confirmed-audit-notification [32] see Alarm and Event Services
    // audit-log-query [33] see Object Access Services
    MaxBacnetConfirmedService = 34,
}

impl From<u8> for ConfirmedServiceChoice {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::AcknowledgeAlarm,
            1 => Self::CovNotification,
            2 => Self::EventNotification,
            3 => Self::GetAlarmSummary,
            4 => Self::GetEnrollmentSummary,
            5 => Self::SubscribeCov,
            6 => Self::AtomicReadFile,
            7 => Self::AtomicWriteFile,
            8 => Self::AddListElement,
            9 => Self::RemoveListElement,
            10 => Self::CreateObject,
            11 => Self::DeleteObject,
            12 => Self::ReadProperty,
            13 => Self::ReadPropConditional,
            14 => Self::ReadPropMultiple,
            15 => Self::WriteProperty,
            16 => Self::WritePropMultiple,
            17 => Self::DeviceCommunicationControl,
            18 => Self::PrivateTransfer,
            19 => Self::TextMessage,
            20 => Self::ReinitializeDevice,
            21 => Self::VtOpen,
            22 => Self::VtClose,
            23 => Self::VtData,
            24 => Self::Authenticate,
            25 => Self::RequestKey,
            26 => Self::ReadRange,
            27 => Self::LifeSafetyOperation,
            28 => Self::SubscribeCovProperty,
            29 => Self::GetEventInformation,
            30 => Self::SubscribeCovPropertyMultiple,
            31 => Self::CovNotificationMultiple,
            32 => Self::AuditNotification,
            33 => Self::AuditLogQuery,
            34 => Self::MaxBacnetConfirmedService,
            _ => panic!("invalid confirmed service choice"),
        }
    }
}

impl ApplicationPdu {
    pub fn encode(&self, buffer: &mut Buffer) {
        match self {
            ApplicationPdu::ConfirmedRequest(req) => req.encode(buffer),
            ApplicationPdu::UnconfirmedRequest(req) => req.encode(buffer),
            ApplicationPdu::ComplexAck(_) => todo!(),
        };
    }

    pub fn decode(reader: &mut Reader) -> Self {
        let byte0 = reader.read_byte();
        let pdu_type: ApduType = (byte0 >> 4).into();
        let pdu_flags = byte0 & 0x0F;
        let _segmented_message = (pdu_flags & PduFlags::SegmentedMessage as u8) > 0;
        let _more_follows = (pdu_flags & PduFlags::MoreFollows as u8) > 0;
        let _segmented_response_accepted =
            (pdu_flags & PduFlags::SegmentedResponseAccepted as u8) > 0;

        match pdu_type {
            ApduType::ConfirmedServiceRequest => {
                let apdu = ConfirmedRequest::decode(reader);
                ApplicationPdu::ConfirmedRequest(apdu)
            }
            ApduType::UnconfirmedServiceRequest => {
                let apdu = UnconfirmedRequest::decode(reader);
                ApplicationPdu::UnconfirmedRequest(apdu)
            }
            ApduType::ComplexAck => {
                let adpu = ComplexAck::decode(reader);
                ApplicationPdu::ComplexAck(adpu)
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub struct ConfirmedRequest {
    pub max_segments: MaxSegments, // default 65
    pub max_adpu: MaxAdpu,         // default 1476
    pub invoke_id: u8,             // starts at 0
    pub sequence_num: u8,          // default to 0
    pub proposed_window_size: u8,  // default to 0
    pub service: ConfirmedRequestSerivice,
}

#[derive(Debug)]
pub struct ComplexAck {
    pub invoke_id: u8,
    pub service: ComplexAckService,
}

impl ComplexAck {
    pub fn decode(reader: &mut Reader) -> Self {
        let invoke_id = reader.read_byte();
        let choice = reader.read_byte().into();

        let service = match choice {
            ConfirmedServiceChoice::ReadProperty => {
                let apdu = ReadPropertyAck::decode(reader);
                ComplexAckService::ReadProperty(apdu)
            }
            ConfirmedServiceChoice::ReadPropMultiple => {
                let apdu = ReadPropertyMultipleAck::decode(reader);
                ComplexAckService::ReadPropertyMultiple(apdu)
            }
            _ => unimplemented!(),
        };

        Self { invoke_id, service }
    }
}

#[derive(Debug)]
pub enum ComplexAckService {
    ReadProperty(ReadPropertyAck),
    ReadPropertyMultiple(ReadPropertyMultipleAck),
    // add more here
}

#[derive(Debug)]
pub enum ConfirmedRequestSerivice {
    ReadProperty(ReadProperty),
    ReadPropertyMultiple(ReadPropertyMultiple),
    // add more here
}

enum PduFlags {
    Server = 0b0001,
    SegmentedResponseAccepted = 0b0010,
    MoreFollows = 0b0100,
    SegmentedMessage = 0b1000,
}

impl ConfirmedRequest {
    pub fn new(invoke_id: u8, service: ConfirmedRequestSerivice) -> Self {
        Self {
            max_segments: MaxSegments::_65,
            max_adpu: MaxAdpu::_1476,
            invoke_id,
            sequence_num: 0,
            proposed_window_size: 0,
            service,
        }
    }

    pub fn encode(&self, buffer: &mut Buffer) {
        let max_segments_flag = match self.max_segments {
            MaxSegments::_0 => 0,
            _ => PduFlags::SegmentedResponseAccepted as u8,
        };

        let control = ((ApduType::ConfirmedServiceRequest as u8) << 4) | max_segments_flag;
        buffer.push(control);
        buffer.push(self.max_segments as u8 | self.max_adpu as u8);
        buffer.push(self.invoke_id);

        // TODO: handle Segment pdu

        match &self.service {
            ConfirmedRequestSerivice::ReadProperty(service) => {
                buffer.push(ConfirmedServiceChoice::ReadProperty as u8);
                service.encode(buffer)
            }
            ConfirmedRequestSerivice::ReadPropertyMultiple(service) => {
                buffer.push(ConfirmedServiceChoice::ReadPropMultiple as u8);
                service.encode(buffer)
            }
        };
    }

    pub fn decode(_reader: &mut Reader) -> Self {
        unimplemented!()
    }
}

#[derive(Debug)]
pub enum UnconfirmedRequest {
    WhoIs(WhoIs),
    IAm(IAm),
}

impl UnconfirmedRequest {
    pub fn encode(&self, buffer: &mut Buffer) {
        buffer.push((ApduType::UnconfirmedServiceRequest as u8) << 4);

        match &self {
            Self::IAm(_) => todo!(),
            Self::WhoIs(payload) => payload.encode(buffer),
        }
    }

    pub fn decode(reader: &mut Reader) -> Self {
        let choice: UnconfirmedServiceChoice = reader.read_byte().into();
        match choice {
            UnconfirmedServiceChoice::IAm => {
                let apdu = IAm::decode(reader).unwrap();
                UnconfirmedRequest::IAm(apdu)
            }
            UnconfirmedServiceChoice::WhoIs => {
                let apdu = WhoIs::decode(reader);
                UnconfirmedRequest::WhoIs(apdu)
            }
            _ => unimplemented!(),
        }
    }
}

pub enum UnconfirmedServiceChoice {
    IAm = 0,
    IHave = 1,
    CovNotification = 2,
    EventNotification = 3,
    PrivateTransfer = 4,
    TextMessage = 5,
    TimeSynchronization = 6,
    WhoHas = 7,
    WhoIs = 8,
    UtcTimeSynchronization = 9,

    // addendum 2010-aa
    WriteGroup = 10,

    // addendum 2012-aq
    CovNotificationMultiple = 11,

    // addendum 2016-bi
    AuditNotification = 12,

    // addendum 2016-bz
    WhoAmI = 13,
    YouAre = 14,

    // Other services to be added as they are defined.
    // All choice values in this production are reserved
    // for definition by ASHRAE.
    // Proprietary extensions are made by using the
    // UnconfirmedPrivateTransfer service. See Clause 23.
    MaxBacnetUnconfirmedService = 15,
}

impl From<u8> for UnconfirmedServiceChoice {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::IAm,
            1 => Self::IHave,
            2 => Self::CovNotification,
            3 => Self::EventNotification,
            4 => Self::PrivateTransfer,
            5 => Self::TextMessage,
            6 => Self::TimeSynchronization,
            7 => Self::WhoHas,
            8 => Self::WhoIs,
            9 => Self::UtcTimeSynchronization,
            10 => Self::WriteGroup,
            11 => Self::CovNotificationMultiple,
            12 => Self::AuditNotification,
            13 => Self::WhoAmI,
            14 => Self::YouAre,
            15 => Self::MaxBacnetUnconfirmedService,
            _ => panic!("invalid unconfirmed service choice"),
        }
    }
}