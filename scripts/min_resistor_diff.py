def analyze_real_world_divider():
    # --- Circuit Parameters ---
    v_in = 3.3  # Input voltage
    r2_nom = 10000.0  # Lower leg fixed resistor (Ohms)
    tolerance = 0.001  # Resistor tolerance (0.1%)
    r_dupont = 0.03  # Estimated DuPont contact resistance (30 mΩ)
    adc_resolution = 12  # ADC resolution (bits)

    # ADC Calculations
    adc_steps = 2**adc_resolution
    lsb_v = v_in / adc_steps  # Volts per ADC step
    lsb_mv = lsb_v * 1000  # Millivolts per ADC step

    # Standard E24 series base values
    e24_base = [
        1.0,
        1.1,
        1.2,
        1.3,
        1.5,
        1.6,
        1.8,
        2.0,
        2.2,
        2.4,
        2.7,
        3.0,
        3.3,
        3.6,
        3.9,
        4.3,
        4.7,
        5.1,
        5.6,
        6.2,
        6.8,
        7.5,
        8.2,
        9.1,
    ]

    # Generate R1 values between 1k and 100k
    r1_values = []
    for multiplier in [1000, 10000]:
        for val in e24_base:
            r1_values.append(val * multiplier)
    r1_values.append(100000.0)

    # Calculate the nominal Vout/Vin ratio for each R1 to find the closest pair
    readings = []
    for r1 in r1_values:
        ratio = r2_nom / (r1 + r2_nom)
        readings.append((r1, ratio))

    # Sort to find the smallest nominal gap
    readings.sort(key=lambda x: x[1])
    min_diff_ratio = float("inf")
    r1_pair = None

    for i in range(len(readings) - 1):
        diff = abs(readings[i + 1][1] - readings[i][1])
        if diff < min_diff_ratio:
            min_diff_ratio = diff
            r1_pair = (readings[i][0], readings[i + 1][0])

    r1_a, r1_b = r1_pair  # Example: 1500 and 1600

    # --- Worst-Case Tolerance Analysis ---
    # Case A: We want to MAXIMIZE the lower voltage reading to close the gap.
    # This happens when R1 is at its minimum (-0.1%) and R2 is at its maximum (+0.1%)
    r1_a_min = (r1_a * (1 - tolerance)) + r_dupont
    r2_max = r2_nom * (1 + tolerance)
    vout_a_max = v_in * (r2_max / (r1_a_min + r2_max))

    # Case B: We want to MINIMIZE the higher voltage reading to close the gap.
    # This happens when R1 is at its maximum (+0.1%) and R2 is at its minimum (-0.1%)
    r1_b_max = (r1_b * (1 + tolerance)) + r_dupont
    r2_min = r2_nom * (1 - tolerance)
    vout_b_min = v_in * (r2_min / (r1_b_max + r2_min))

    # Calculate the worst-case voltage gap
    worst_case_diff_v = abs(vout_b_min - vout_a_max)
    worst_case_diff_mv = worst_case_diff_v * 1000
    worst_case_adc_gap = worst_case_diff_v / lsb_v

    # --- Output formatting ---
    print("--- Circuit Parameters ---")
    print(f"Input Voltage:    {v_in}V")
    print(f"ADC Resolution:   {adc_resolution}-bit ({adc_steps} discrete steps)")
    print(f"ADC Sensitivity:  {lsb_mv:.3f} mV per step")
    print(f"Resistor Tol:     {tolerance * 100}%")
    print(f"DuPont Connector: {r_dupont * 1000:.0f} mΩ\n")

    print("--- Worst-Case Analysis ---")
    print(f"Closest E24 Step: {r1_a:.0f} Ω vs {r1_b:.0f} Ω")
    print(f"Worst-Case Gap:   {worst_case_diff_mv:.2f} mV")
    print(f"ADC Steps Clear:  {worst_case_adc_gap:.1f} steps\n")

    if worst_case_adc_gap > 3:
        print(
            "CONCLUSION: Safe! You have enough margin to reliably measure this difference."
        )
    else:
        print(
            "CONCLUSION: Warning! The difference may be indistinguishable due to noise or tolerances."
        )


if __name__ == "__main__":
    analyze_real_world_divider()
