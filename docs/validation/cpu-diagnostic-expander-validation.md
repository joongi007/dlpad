# CPU 진단·MPD header 펼침 상태 실제 GUI

2026-10-03 Linux private Xvfb/DBus의 CPU SHA `608eeeb5a2f3a046e0a389ca788770f58e6ff38369d056ef8c8debdb262829c9` /12,403,064B snapshot이다. 이 결과는 이후 source 또는 다른 OS를 검증한 것으로 대체하지 않는다. root는 별도 private 화면에서 PLAN을 검증하며 본 시험은 DIAGNOSTICS/MPD 두 header에 한정한다.

## 실제 상태 전환

normal 합성 tool을 실제 argv/process worker로 실행해 Failed job(revision3/Attempt1)을 만들었다. DLPAD_TEST_REPORT_STRESS는 제거했으며 큰 report builder를 사용하지 않았다. role43 `실패 원인 보기`의 실제 GetState를 조회하고 visible native Action.DoAction으로 closed/open/closed/reopen을 수행했다. 이후 작은 소유 로컬 MPD를 actual attachment worker로 첨부하고 role43 `MPD 0`의 같은 전환을 확인했다.

| header | 접힘 uint32 | 펼침 uint32 | Expandable bit9 | Expanded bit10 순서 |
|---|---:|---:|---|---|
| 실패 원인 보기 | 1124076288 | 1124077312 | 항상true | false/true/false/true |
| MPD 0 | 1124076288 | 1124077312 | 항상true | false/true/false/true |

locked atspi-common State enum에 따라 bits를 해석했다. 접힘은 Expandable=true/Expanded=false이며 별도 Collapsed bit를 관측했다고 주장하지 않는다. `실패 원인 보기` extents(8,503,109,32), `MPD 0`(26,437,62,32)의 중심은 실제 content clip 안이다. hidden action 또는 clip 확대를 사용하지 않았다. 실제 mpd-3.png에서 펼쳐진 BaseUrl/Initialization 및 range0-20 구조 본문을 직접 확인했다. screenshot만으로 metadata 성공을 판단하지 않았다.

## 원 작업·입력과 종료 검증

진단 열람/attachment와 header 전환 전후 Failed snapshot(ID/revision3/Attempt1/options/argv)이 같았다. 보존 SQLite job_json과 최초 관측이 정확히 같고 job/attempt 각1개, integrity_check=ok였다. 로컬 MPD before/after SHA `c76828ed5123b9804b3638dc066035dbc56f792e23049d3a65094dc285bf72ce`도 동일했다. MPD URL은 합성 fixture이고 network 요청을 하지 않았다.

정상 창 닫기 exit0, 소유 app/bus 채널 회수, binary before/after SHA 동일. 기록된 PID/start identity가 host /proc에 남지 않았으며 이번 고유 tmp runtime prefix를 사용하는 process가 없음을 직접 확인했다. independent-verification.json에 native bits sequence/DB/source/identity 검증을 저장했다. 기존8h PID/다른 agent private 환경에 접근하거나 종료하지 않았다.

## 실행과 근거

`python3 scripts/cpu_diagnostic_expander_probe.py --binary target/release/dlpad-cpu --output docs/validation/evidence/cpu-diagnostic-expander/current-1 --expected-sha 608eeeb5a2f3a046e0a389ca788770f58e6ff38369d056ef8c8debdb262829c9` → exit0/PASS.

[evidence/cpu-diagnostic-expander/current-1/](evidence/cpu-diagnostic-expander/current-1/)의 result.json/state decimal arrays/PNG/jobs.sqlite/independent-verification.json/bounded logs를 따른다. 전용 harness SHA `042f1a54c9f1952c90629e4f7b17a4bc974e37bd94dba72816a8d6d8670ab260`. 기존 cpu_report_stress_probe의 widget/scroll_outer/state를 readonly import했다. 사용자 desktop·clipboard·실미디어·auth·외부 site는 사용하지 않았다.

제품/shared helper/Cargo를 수정하지 않았으며 전용 Python 구문 검사와 문서 검사를 실행했다. root가 제품 통합 검증을 별도로 수행한다.

## 남은 범위

Windows/macOS native header 상태, 실제 Orca 사용자 경험/음성, animation 중간 frame별 관측, 최대report 및 전체S10/P0 릴리스는 본 한 건의 stable 상태 시험으로 완료하지 않는다. 기존166946 입력/대형report 검증과 최신608eeeb metadata 검증의 snapshot을 구분한다.
