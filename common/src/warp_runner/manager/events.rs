use warp::{
    blink::BlinkEventKind,
    logging::tracing::log,
    multipass::MultiPassEventKind,
    raygun::{MessageEventKind, RayGunEventKind},
};

use crate::{
    warp_runner::{
        conv_stream,
        manager::commands::handle_blink_cmd,
        ui_adapter::{self, did_to_identity, MultiPassEvent},
        RayGunCmd, WarpCmd, WarpEvent,
    },
    WARP_EVENT_CH,
};

use super::{
    commands::{
        handle_constellation_cmd, handle_multipass_cmd, handle_other_cmd, handle_raygun_cmd,
    },
    MultiPassCmd,
};

pub async fn handle_multipass_event(
    evt: Option<MultiPassEventKind>,
    warp: &mut super::Warp,
) -> Result<(), ()> {
    let evt = match evt {
        Some(e) => e,
        None => return Ok(()),
    };
    log::debug!("received multipass event: {:?}", &evt);
    let warp_event_tx = WARP_EVENT_CH.tx.clone();
    match ui_adapter::convert_multipass_event(evt, &mut warp.multipass, &mut warp.raygun).await {
        Ok(evt) => {
            if let Err(e) = warp_event_tx.send(WarpEvent::MultiPass(evt)) {
                log::error!("failed to send warp_event: {e}");
                return Err(());
            }
        }
        Err(e) => {
            log::error!("failed to convert multipass event: {}", e);
        }
    }

    Ok(())
}

pub async fn handle_raygun_event(
    evt: Option<RayGunEventKind>,
    warp: &mut super::Warp,
    stream_manager: &mut conv_stream::Manager,
) -> Result<(), ()> {
    let evt = match evt {
        Some(e) => e,
        None => return Ok(()),
    };
    log::debug!("received raygun event: {:?}", &evt);
    let warp_event_tx = WARP_EVENT_CH.tx.clone();
    match ui_adapter::convert_raygun_event(
        evt,
        stream_manager,
        &mut warp.multipass,
        &mut warp.raygun,
    )
    .await
    {
        Ok(evt) => {
            if let Err(e) = warp_event_tx.send(WarpEvent::RayGun(evt)) {
                log::error!("failed to send warp_event: {e}");
                return Err(());
            }
        }
        Err(e) => {
            log::error!("failed to convert raygun event: {}", e);
        }
    }

    Ok(())
}

pub async fn handle_message_event(
    evt: Option<MessageEventKind>,
    warp: &mut super::Warp,
) -> Result<(), ()> {
    let msg = match evt {
        Some(e) => e,
        None => return Ok(()),
    };
    let warp_event_tx = WARP_EVENT_CH.tx.clone();
    match ui_adapter::convert_message_event(msg, &mut warp.multipass, &mut warp.raygun).await {
        Ok(evt) => {
            if let Err(e) = warp_event_tx.send(WarpEvent::Message(evt)) {
                log::error!("failed to send warp_event: {e}");
                return Err(());
            }
        }
        Err(e) => {
            log::error!("failed to convert message event: {}", e);
        }
    }

    Ok(())
}

// currently there's no need for warp runner to respond to blink events. all the other handle_x_event functions send forward the event over the WARP_EVENT_CH
// this function does the same.
pub async fn handle_blink_event(
    evt: BlinkEventKind,
    _warp: &mut super::Warp,
) -> anyhow::Result<()> {
    let warp_event_tx = WARP_EVENT_CH.tx.clone();
    warp_event_tx.send(WarpEvent::Blink(evt))?;
    Ok(())
}

pub async fn handle_warp_command(
    evt: Option<WarpCmd>,
    warp: &mut super::Warp,
    stream_manager: &mut conv_stream::Manager,
) -> Result<(), ()> {
    let cmd = match evt {
        Some(e) => e,
        None => return Ok(()),
    };
    if !matches!(cmd, WarpCmd::RayGun(RayGunCmd::SendEvent { .. })) {
        log::debug!("WARP CMD: {}", &cmd);
    } else {
        // the SendEvent is mostly just the typing indicator
        log::trace!("WARP CMD: {}", &cmd);
    }

    let warp_event_tx = WARP_EVENT_CH.tx.clone();
    match cmd {
        WarpCmd::Other(cmd) => {
            // this one could be parallelized
            handle_other_cmd(cmd).await;
        }
        WarpCmd::Tesseract(_cmd) => {
            // not accepted at this stage of the program. do nothing and drop the rsp channel
        }
        WarpCmd::MultiPass(cmd) => {
            // if a command to block a user comes in, need to update the UI
            // todo: handle block events
            if let MultiPassCmd::Block { did, .. } = &cmd {
                if let Ok(ident) = did_to_identity(did, &warp.multipass).await {
                    if let Err(e) =
                        warp_event_tx.send(WarpEvent::MultiPass(MultiPassEvent::Blocked(ident)))
                    {
                        log::error!("failed to send warp_event: {e}");
                        return Err(());
                    }
                }
            }
            if let MultiPassCmd::Unblock { did, .. } = &cmd {
                if let Ok(ident) = did_to_identity(did, &warp.multipass).await {
                    if let Err(e) =
                        warp_event_tx.send(WarpEvent::MultiPass(MultiPassEvent::Unblocked(ident)))
                    {
                        log::error!("failed to send warp_event: {e}");
                        return Err(());
                    }
                }
            }
            handle_multipass_cmd(cmd, warp).await;
        }

        WarpCmd::RayGun(cmd) => {
            handle_raygun_cmd(cmd, stream_manager, &mut warp.multipass, &mut warp.raygun).await
        }

        WarpCmd::Constellation(cmd) => handle_constellation_cmd(cmd, &mut warp.constellation).await,
        WarpCmd::Blink(cmd) => handle_blink_cmd(cmd, &mut warp.blink).await,
    }
    Ok(())
}
