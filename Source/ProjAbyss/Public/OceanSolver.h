#pragma once

#include "CoreMinimal.h"

/**
 * FGerstnerWave
 * Represents a single wave layer. We stack these to create complex ocean surfaces.
 * Deep swells = High Wavelength, High Amplitude.
 * Surface chop = Low Wavelength, Low Amplitude.
 */
struct FGerstnerWave
{
    float Wavelength;
    float Amplitude;
    float Speed;
    FVector2D Direction;
    float Steepness; // 0.0 to 1.0 (How "sharp" the peak is)

    FGerstnerWave(float InLength, float InAmp, float InSpeed, FVector2D InDir, float InSteep)
        : Wavelength(InLength), Amplitude(InAmp), Speed(InSpeed), Direction(InDir), Steepness(InSteep)
    {
        Direction.Normalize();
    }

    float GetFrequency() const { return (2.0f * PI) / Wavelength; }
    float GetPhaseConstant() const { return Speed * GetFrequency(); }
};

/**
 * FOceanSolver
 * The CPU-side physics calculator.
 * Usage: Create one instance in your GameState, Update 'Time', then ask it for heights.
 */
struct FOceanSolver
{
public:
    float Time;
    TArray<FGerstnerWave> Waves;

    FOceanSolver() : Time(0.f)
    {
        // DEFAULT WAVE PROFILE (Week 1 Setup)
        // 1. Big Swell (North)
        Waves.Add(FGerstnerWave(6000.f, 150.f, 400.f, FVector2D(1.0f, 0.2f), 0.4f));
        // 2. Medium Chop (North-East)
        Waves.Add(FGerstnerWave(3500.f, 80.f, 250.f, FVector2D(0.7f, 0.7f), 0.6f));
        // 3. Small Detail (East)
        Waves.Add(FGerstnerWave(1500.f, 40.f, 350.f, FVector2D(0.2f, 1.0f), 0.8f));
    }

    /** * Calculates the Z-Height of the water surface at a given world X,Y.
     * Note: This is an approximation (Sum of Sines).
     * True Gerstner waves displace X/Y too, but for buoyancy, this is 99% accurate enough
     * and much faster than iterative solving.
     */
    float GetWaveHeightAt(const FVector& Location) const
    {
        float TotalOffsetZ = 0.f;

        for (const FGerstnerWave& Wave : Waves)
        {
            float Freq = Wave.GetFrequency();
            float Phase = Wave.GetPhaseConstant() * Time;

            // Dot product gets the distance along the wave's direction
            float ProjectedPos = (Location.X * Wave.Direction.X) + (Location.Y * Wave.Direction.Y);

            // The Wave Equation: A * sin(k * x - w * t)
            // We multiply by Steepness for that "sharp peak" look
            TotalOffsetZ += (Wave.Amplitude * Wave.Steepness) * FMath::Sin((Freq * ProjectedPos) + Phase);
        }

        // Return the global sea level (Z=0) + the wave offset
        return TotalOffsetZ;
    }
};