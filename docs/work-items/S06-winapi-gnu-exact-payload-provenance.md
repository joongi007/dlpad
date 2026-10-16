# S06: Winapi GNU 원 payload provenance 추적

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R05, R11
- 담당/수정 범위: root 전용문서/증거·고정notice mapping조정; diagnostic_cache /tmp 원공개API/캐시 비교만
- 선행 작업: Windows cpu normal/build feature259 graph와원Cargo.lock0.4.0 checksum 확인

## 사용자 결과

배포하는Windows importlib package의원license/source를정확히연결한다. 다른버전의grant를근거없이소급하지않는다.

## 계약과 범위

기존cached crate0.4.0 원SHA와1416 .a byte/size/GitblobSHA1을boundedGitHub원commit tree와대조한다. manifest/src/build 원blob이맞는publish근처commit을먼저찾고그subtree만확인한다. 원crates.io created_at 및공개GitHubtag/history는인증없이무료HTTPS읽기만한다. 2MiB tree한도를초과한응답은실패보존하고작은subtreequery를사용한다. 기존프로젝트.git/remote/계정인증/외부메시지/업로드/소스rewrite를하지않는다. mapping/schema/Cargo공유변경은root만담당한다.

## 인수 조건

exact1416payload+Cargo.orig/src/build 원Gitblob/size가일치하는commit·license bytes/sha/pathscope를검토한뒤만매핑후보를승격한다. 생성된def/source와embeddednotice 검수는별도이며 SPDX선언·같은version문자열만으로전체완료를표현하지않는다. 실패raw/미확정범위와후속결정을남긴다.

## 검증과 handoff

0.3.9 원commit796a... tree와0.4.0 cached1416lib 대조는1개일치/1415개불일치, manifest/src/build도불일치라grant binding으로채택하지않았다. 정확publish근처commit을후속조회한다. 프로젝트/Cargo/lock수정0.

후속 공급 후보: 공식 commit9497609ef44cc9bcd16cd2411c0ee6ccaf5483aa의1416 import library 및 manifest/build/source가 cached0.4.0 payload와 전체 Git blob/크기로 일치하는 초안을 확보했다. 같은 commit MIT/Apache 원grant를 검토 중이다. 이 commit은 원발행일 뒤이므로 발행 당시 revision 확정으로 표현하지 않는다. root의 독립 원자료 검토와 엄격 payload binding 구현은 남아 있으며 기존 VCS 메타데이터를 생성하지 않는다. 생성 원본/embedded notice 완비도 미검증이다.

root 통합: 정확 Winapi 사례만 별도payload_binding을 허용하고 원Cargo.lock checksum·원crate 전체1420파일·1416 import archive·고정Git subtree 및 원license Gitblob을 연결했다. 기존VCS필수 정책은 다른package에 그대로 유지한다. 실제 root 검증은원positive와wrongchecksum/count/tree/원Lock/원grant+claimedSHA동시변조5실패거절 PASS다. [실제 결과](../validation/evidence/winapi-gnu-exact-binding-root-20261005-continue/result.json). 기존cache 변경0·가짜VCS생성0. 반복 가능한 scripts/windows_winapi_payload_binding_probe.py를 추가했으며 --help도 실행했다. 독립 통합검토·최신candidate 전체SBOM 재생성은 후속이다. 전체notice/법적완비와release_ready는 여전히 false다.

독립 통합 검토에서 차단 결함은 발견되지 않았고 정확binding의 범위 제한을 확인했다. 캐시 자체 변조·unknown/symlink/missing과 원crate/tree 파일 변조의 소유fixture 회귀는 후속으로 남긴다. 현재 원archive SHA는 검증하지만 gzip trailer CRC 전체소비를 별도확인했다고 주장하지 않는다. 전체coverage는 false이며 최신candidate 갱신은 최신CPU 전체검증 뒤 진행한다.

scoped 완료: 원payload/grant binding을 통합했고 실제 소유fixture7변조거절과 원1424파일 size/SHA 보존을 root가 원receipt·helper SHA와 대조했다. 원unknown case의 선행count상한 때문에 첫collector 기대문구가 실패한 기록은 보존했으며 수정 후 실제7case가PASS다. [원7case](../validation/evidence/winapi-gnu-exact-binding-root-20261005-continue/owned-tamper-result.json). fullSBOM/생성원본/embeddedlegal/공식runtime공급 및 공개릴리스 인수는 별도이며 완료표시하지 않는다.
