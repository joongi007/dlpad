# S04: Managed runtime private GUI collector

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당/수정 범위: diagnostic_cache; scripts/cpu_runtime_tools_probe.py, 이 문서만
- 선행 작업: root runtime tool selector/typed selected field integration; frozen binary SHA

## 사용자 결과

빈 관리 root의 세 도구 전환과 runtime 자동 설치 disabled를 관찰한다. 합성 관리 Deno/FFmpeg의 명시 선택/사용이 정확한 다운로드 필드에 전달되는지, wrong version에서 prior active가 보존되는지 검사한다.

## 계약과 범위

Linux owned Xvfb/private D-Bus/tempXDG 및 합성 shebang만 사용. 사용자의 desktop/clipboard/외부 도구/네트워크 설치 없음. 기존 scripts는 read-only helper 재사용한다. Binary SHA 전후 확인, 로그 bounded helper, 노드256/스크롤16/관측15초/결과64KiB. 작업 raw 새output 생성 및 owned PID 정리.

## 인수 조건

empty: yt-dlp/Deno/FFmpeg·ffprobe 각각 refresh/empty; 두 runtime Install disabled/pending 안내; yt-dlp 복귀시 Install enabled/pending 또는 stale failure 없음. select-use: synthetic 두버전 metadata SHA-size/executable role와실제version argv검사→v2active→Use→Deno/FFmpeg 올바른ENTRY 값. wrong-version: v2version실행 mismatch→고정 failure→v1active/bytes 및외부sentinel 보존. 실제 공급·license·codec 검증으로 확대하지 않는다.

## 검증과 handoff

작성 후 AST/validate-only. 실제 GUI는root 최신release에서통합담당 실행. GUI raw성공을사전주장하지않는다.

AST parse PASS / --validate-only exit0(no processes started). root catalog version mismatch 메시지와typedToolId→field 계약을읽기확인했다. 실제GUI/합성자식버전실행은아직미실행, source READY/frozen.

## 실제 첫 실행 결과

root가 최신releaseSHA c956993a9c46ab22793cc74aa9ceeef837cab277f6c5440f3764efcfef965354로emptycase를실행했다. runtime-tools-linux-empty-first/result.json은exit1/bounded widget lookup failed: 도구이며ownedcleanuptrue, desktopfallbackfalse, 원binarySHA불변이다. root가failure.png를직접확인해native도구nav가실제보이는것을확인했다. 수집기widgetrole/action검색을수정중이며GUI인수PASS로표시하지않는다.

첫 empty 실행은 visible 도구 nav를 찾지 못했다. 기존 cpu_options_probe.lookup은 download 전용으로 y>=65/다운로드 시작 footer 존재를 요구한다. 도구 nav는 상단 y45이고 tools panel에는 그footer가없으므로재사용계약이맞지않았다. 전용수집기만fullownedwindow bounds 및freshGetRole/GetNActions를검증하는boundedlookup으로교정했다. 실패시AT-SPIraw snapshot<=1MiB를별도보존하고정확role/action/bounds를관찰가능하게했다. 원runtime-tools-linux-empty-first raw는변경하지않았다. AST PASS, 실제재실행대기.

## role 재시험 실제 결과

empty-role-retry/select-use-role-first/wrong-version-role-first 세 collector 모두 도구lookup에서exit1이며ownedcleanuptrue. root가failure-atspi.json을직접읽어 도구node 실제GetRole43(pushbutton), Text interface없음을확인했다. Action.GetNActions를click필수로사용한수집기조건은nativeadapter실제노출과맞지않아 추가보정한다. pointer시험은actualbuttonrole43·enabled·원ownedwindowextents에한정하며labelrole29를대상에서제외한다. 세실패원시를유지하고제품GUI완료로표시하지않는다.

## 빈 목록 격리 실제 인수

runtime-tools-linux-empty-button-retry/result.json은exit0/PASS,yt-dlp→Deno→FFmpeg·ffprobe→yt-dlp4전환의runtimepending및installdisabled/빈목록/이전오류없음을확인했다. root가empty-3.png를직접열어FFmpeg선택상태와비활성설치·pending안내를대조했다. 외부sentinel보존/ownedcleanuptrue/desktopfallbackfalse,원binarySHA일치. 정상선택·잘못된version거절두case는button-retry새원시에서실행중이며아직GUI완료로확대하지않는다. runtimetab의yt-dlp전용help/license혼동은독립감사후속으로기록했다.

button-retry select-use/wrong-versionは自動refresh中の次refreshをdisabledとして失敗した。通常busyを製品不具合と判定しないようclickは15秒上限でfreshGetState Enabledを待ち、last_action_waitに具体name/role/action/state/boundsを保存する。時間超過時は待ったnameをerrorに含める。旧raw保存/timeout拡大なし。次GUIはroot新binary以後。

## 새 설명 바이너리의 enabled 재시험

empty-enabled-retry는exit0/PASS로4tool전환·runtimepending/설치disabled를최신runtime전용설명에서확인했다. select-use/wrong-version-enabled-retry는exit1로첫activev1행의disabled 이 버전 활성화 버튼을수집기가반복선택했다. 같은label의inactivev2enabled버튼을모두탐색하도록후속교정한다. 정상선택/버전거절은아직GUI완료로표시하지않는다. 제품Clippy와release는exit0.

## 중복 버튼 이후 수집기 보정

select-use/wrong-version-duplicate-retry는수집기진단dict의state중복keyword로exit1. 주담당이dict(node,name=name,state=state)로수정하여중복값을명시덮어쓰고state-retry새증거에서재실행했다. 제품Rust/binary는변경하지않고실패원시는보존했다. Windowscurrentworkspace4명령및sourcehash380개대조PASS는별도검증이다.

## 최종 scoped 실제 결과

select-use-state-retry/wrong-version-state-retry 두실제collector는exit0/PASS. binarySHA3877868b3e07d7562525f133d07eb04491e79277847126f2837f960fe90aec9a전후동일. 합성Deno/FFmpegversion2.0.0 실제실행후active전환·Use→각namedENTRYexactpath readback, 잘못된version에서prior1.0.0active 및원files보존을확인했다. root가deno-used.png/ffmpeg-used.png/wrong-version final.png를직접대조했다. 외부sentinel/ownedcleanuptrue/desktopfallbackfalse. 자동공급설치·Windows/macOS관리GUI·관리실행lease·모든OSrelease완료를뜻하지않는다.
