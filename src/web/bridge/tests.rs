use super::*;

fn render_page(bootstrap: &str, candidate_count: usize) -> BridgePage {
    render(
        "proxy.example.com",
        bootstrap,
        2 * 1024 * 1024,
        32 * 1024 * 1024,
        16 * 1024,
        1024,
        true,
        candidate_count,
        [3, 5, 8, 12],
        25,
        10,
        90,
        15,
        15,
        120,
        0,
        &SecureRandom::new(),
    )
}

#[test]
fn rendered_page_contains_bounded_negotiation_contract() {
    let page = render_page("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", 4);
    assert!(!page.body.contains("__"));
    assert!(!page.body.contains("bridge="));
    assert!(page.body.contains("X-Carrier-Capabilities"));
    assert!(page.body.contains("X-Carrier-Attempt"));
    assert!(page.body.contains("candidateCount=4"));
    assert!(page.body.contains("candidateDeadlines=[3,5,8,12]"));
    assert!(page.body.contains("X-Up-Seq"));
    assert!(page.body.contains("X-Lane-ID"));
    assert!(page.body.contains("tproxy-auto-v1."));
    assert!(page.body.contains("tproxy-auto-lane-v1."));
    assert!(page.body.contains("globalThis.TelemtBridgeResponse"));
    assert!(page.body.contains("globalThis.TelemtBridgeRequest"));
    assert!(page.body.contains("globalThis.TelemtBridgeBuffers"));
    assert!(page.body.contains("globalThis.TelemtBridgeRecovery"));
    assert!(page.body.contains("responseBody.read"));
    assert!(!page.body.contains("arrayBuffer()"));
    assert!(page.body.contains("maxChunks=4096"));
    assert!(
        page.content_security_policy
            .contains("frame-ancestors http://127.0.0.1:*")
    );
}

#[test]
fn rendered_page_preserves_the_ios_bootstrap_literal() {
    let bootstrap = "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB";
    let page = render_page(bootstrap, 2);
    assert!(
        page.body
            .contains(&format!("let bootstrap=\"{bootstrap}\""))
    );
}

#[test]
fn rendered_page_embeds_the_configured_bridge_timing_policy() {
    let page = render(
        "proxy.example.com",
        "GGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG",
        2 * 1024 * 1024,
        32 * 1024 * 1024,
        16 * 1024,
        1024,
        true,
        4,
        [3, 5, 8, 12],
        17,
        7,
        41,
        13,
        11,
        119,
        4,
        &SecureRandom::new(),
    );

    assert!(page.body.contains("let longPollMs=17*1000"));
    assert!(page.body.contains("bridgeRequestMs=7*1000"));
    assert!(page.body.contains("bridgeRetryMs=41*1000"));
    assert!(page.body.contains("bridgeRecoveryMs=13*1000"));
    assert!(page.body.contains("websocketOpenMs=11*1000"));
    assert!(page.body.contains("reconnectGraceMs=119*1000"));
    assert!(page.body.contains("let probeCoalesceMs=4"));
    assert!(
        page.body
            .contains("helloTimer=setTimeout(()=>fail('timeout'),bridgeRequestMs)")
    );
    assert!(page.body.contains(
        "if(!createStarted){createStarted=true;if(helloTimer)clearTimeout(helloTimer);helloTimer=null;helloFrame=message.data"
    ));
}

#[test]
fn effective_deadline_formula_uses_the_final_checkpoint() {
    let page = render_page("CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC", 3);
    assert!(
        page.body
            .contains("negotiatedFinalDeadline=candidateDeadlines[3]")
    );
    assert!(
        page.body
            .contains("carrierAttempt>=negotiatedCandidateCount?negotiatedFinalDeadline")
    );
}

#[test]
fn disabled_negotiation_does_not_arm_a_carrier_deadline() {
    let page = render(
        "proxy.example.com",
        "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD",
        2 * 1024 * 1024,
        32 * 1024 * 1024,
        16 * 1024,
        1024,
        false,
        1,
        [3, 5, 8, 12],
        25,
        10,
        90,
        15,
        15,
        120,
        0,
        &SecureRandom::new(),
    );
    assert!(page.body.contains(
        "if(negotiationEnabled){negotiationStartedAt=Date.now();armCarrierDeadline(attemptEpoch)}"
    ));
    assert!(
        page.body
            .contains("negotiationEnabled?'tproxy-auto-v1.':'tproxy-v1.'")
    );
}

#[test]
fn retry_and_attempt_state_are_frozen_before_fetch() {
    let page = render_page("EEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE", 4);
    assert!(
        page.body
            .contains("async function send(path,frozenOptions,remainingBudget,maxAttempts)")
    );
    assert!(!page.body.contains("makeOptions"));
    assert!(page.body.contains(
        "if(settings.closed()||(external&&external.aborted))throw new Error('request aborted')"
    ));
    assert!(page.body.contains(
        "const frozen=options('POST',bootstrap,snapshot.hello,attemptHeaders(snapshot.attempt,snapshot.failure),controller.signal)"
    ));
}

#[test]
fn ambiguous_commit_is_resolved_before_carrier_advance() {
    let page = render_page("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", 4);
    assert!(
        page.body
            .contains("if(snapshot.selected){advanceConfirmed(reason,epoch);return}")
    );
    assert!(page.body.contains("resolveAttempt(reason,epoch,snapshot)"));
    assert!(page.body.contains(
        "sessionEcho(response,snapshot.attempt,['provisional','committed','healthy'],true)"
    ));
    assert!(
        page.body
            .contains("if(echo.state!=='provisional'){switching=false;fail('protocol');return}")
    );
    assert!(page.body.contains("const token=cleanupToken||sessionToken"));
    assert!(page.body.contains("'X-Carrier-Failure':terminalFailure"));
    assert!(
        page.body
            .contains("addEventListener('pagehide',()=>fail('navigation')")
    );
}

#[test]
fn rendered_page_preserves_exact_v1_status_control_envelope() {
    let page = render_page("HHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHH", 4);

    assert_eq!(
        page.body
            .matches("port.postMessage({t:'status',state})")
            .count(),
        1
    );
    assert!(!page.body.contains("port.postMessage({t:'status',state,"));
}
