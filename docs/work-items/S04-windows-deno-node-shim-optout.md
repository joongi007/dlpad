# S04: Windows Deno node shim opt-out 실제 인수

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당/수정 범위: diagnostic_cache; 신규 scripts/windows_deno_node_shim_optout_probe.py, 본 문서 및 fresh evidence만
- 선행 작업: 고정 공식 exe/source SHA, node shim 원인 및 공식 DENO_DISABLE_NODE_SHIM 근거

## 사용자 결과

fresh 원 소유 executable copy와 DENO_DIR에서 opt-out이 원 실행 파일 link count를 유지하고 cache32MiB를 만족하는지 실제 Windows CLI로 확인한다.

## 계약과 범위

원 공급 exe 읽기만, fresh copy create-only/fullSHA 대조. 새 수집기 내 embedded C# original readable HANDLE의 FILE_STANDARD_INFO 및 FILE_ID_INFO로 native nlink/identity beforeafter를 수집한다. original private CLI host/Job 사용, no-config/no-lock/no-prompt/deny3flags/import0/network0. typed child env DENO_DISABLE_NODE_SHIM=1은 이번 collector 환경만 설정한다. 기존 collector/Rust/Cargo/공유 파일 변경 없음.

## 인수 조건

원 exe SHA 불변, fresh native nlink1 전후·같은 nativeID, version2.9.7, pure 계산/deniedquery/read거부, node.exe cache 부재, 논리 cache≤32MiB, private input desktop·source 불변. actual EJS/Node 호환·제품 guardian 환경 전파는 후속이다.

## 검증과 handoff

작성 중. 근거: https://github.com/denoland/deno/blob/0c071246a412575e07423263404a5d13e7ed6aa2/cli/node_compat_shim.rs 및 https://docs.deno.com/runtime/fundamentals/node/ .

## 실제 완료 범위

`python3 -m py_compile scripts/windows_deno_node_shim_optout_probe.py` exit0. 실제 `python3 scripts/windows_deno_node_shim_optout_probe.py /mnt/d/project/dlpad/docs/validation/evidence/windows-deno-node-shim-optout-first` session82071 terminal exit0/PASS, 소유 핸들 종료 확인. 원 C# compile/native before/version/pure/native after 5명령 전부 exit0/private-desktop/input-desktop-unchanged/source_unchanged이다. 모든 operation source_after를 당시 현재 root snapshot과 독립 대조하여 일치했다. 이 검토 뒤 root 후속 fixture 소스 게시와는 구분한다.

원 native FILE_STANDARD_INFO bytes97,462,048/nlink1이 전후 같고 FILE_ID_INFO volume13266031973415618410/fileID13140400000024000000000000000000도 같다. source와 freshcopy 전후 SHA는 고정 e020f3e232bd16e33768dee528e5983349c962952051ced0a5d58ad42f5d9b33 그대로다. cache10entries/논리250,768 B로32MiB 이하이며 node_compat_bin/node.exe는 없다. 합성 계산5000050000, read/net/env query denied, owned sentinel read 거부 true 및 원 sentinel 보존을 확인했다.

명령 outer elapsed compile3.45초/nativebefore4.13초/version3.54초/pure3.48초/nativeafter3.47초는 host와 snapshot 비용을 포함한다. Deno 단독 latency/peak RSS는 측정하지 않았고 peak_rss=null이다. 이 PASS는 node shim opt-out과 fresh cache의 bounded offline 기능에 한정한다. actual EJS/Node 호환, 관리 ToolManager 설치/선택/정리 통합 및 제품 guardian child env 전파, 공급 trust/전체 notice/source/P0는 미검증이다. 기존 source nlink3/cache는 삭제하거나 변경하지 않았다.
